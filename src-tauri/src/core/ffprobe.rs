use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde::Deserialize;

use crate::core::error::{AppError, AppResult};
use crate::models::MediaProbeInfo;

#[derive(Debug, Deserialize, Default)]
struct FfprobeOutput {
    format: Option<FfprobeFormat>,
    streams: Vec<FfprobeStream>,
}

#[derive(Debug, Deserialize, Default)]
struct FfprobeFormat {
    #[serde(rename = "duration")]
    duration: Option<String>,
    #[serde(default)]
    tags: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Default)]
struct FfprobeStream {
    #[serde(rename = "codec_type")]
    codec_type: Option<String>,
    #[serde(rename = "codec_name")]
    codec_name: Option<String>,
    width: Option<i32>,
    height: Option<i32>,
    #[serde(rename = "r_frame_rate")]
    r_frame_rate: Option<String>,
    #[serde(rename = "avg_frame_rate")]
    avg_frame_rate: Option<String>,
    #[serde(default)]
    tags: HashMap<String, String>,
}

pub fn probe_media(path: &Path) -> AppResult<MediaProbeInfo> {
    let ffprobe_exe = find_ffprobe()?;

    let mut command = Command::new(ffprobe_exe);
    command
        .args([
            "-v",
            "error",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
        ])
        .arg(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = run_ffprobe_with_timeout(&mut command, Duration::from_secs(30))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::FfprobeUnavailable(format!(
            "ffprobe 失败: {}",
            stderr
        )));
    }

    let parsed: FfprobeOutput = serde_json::from_slice(&output.stdout)?;

    let duration_ms = parsed
        .format
        .as_ref()
        .and_then(|f| f.duration.as_ref())
        .and_then(|d| d.parse::<f64>().ok())
        .map(|s| (s * 1000.0) as i64);
    let capture_time = capture_time_from_tags(&parsed.format, &parsed.streams);

    let video_stream = parsed
        .streams
        .iter()
        .find(|s| s.codec_type.as_deref() == Some("video"));

    let (width, height, fps, codec) = match video_stream {
        Some(s) => {
            let fps = s
                .r_frame_rate
                .as_ref()
                .or(s.avg_frame_rate.as_ref())
                .and_then(parse_frame_rate);
            (s.width, s.height, fps, s.codec_name.clone())
        }
        None => (None, None, None, None),
    };

    Ok(MediaProbeInfo {
        duration_ms,
        width,
        height,
        fps,
        codec,
        capture_time,
    })
}

/// Waits without leaving a timed-out ffprobe child behind. This mirrors the
/// FFmpeg derivative supervisor: polling keeps the child handle available so
/// it can be terminated and reaped before the scan proceeds.
fn run_ffprobe_with_timeout(
    command: &mut Command,
    timeout: Duration,
) -> AppResult<std::process::Output> {
    let mut child = command.spawn()?;
    let deadline = Instant::now() + timeout;
    loop {
        if child.try_wait()?.is_some() {
            return child.wait_with_output().map_err(AppError::from);
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(AppError::FfprobeUnavailable(
                "ffprobe 超时（30 秒）；已终止子进程".to_string(),
            ));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// FFmpeg's interoperable metadata key is the RFC 3339 creation_time. Some
/// cameras instead expose a vendor QuickTime key; all other ambiguous date
/// tags are deliberately ignored rather than guessing a timezone.
fn capture_time_from_tags(
    format: &Option<FfprobeFormat>,
    streams: &[FfprobeStream],
) -> Option<i64> {
    format
        .as_ref()
        .map(|entry| &entry.tags)
        .into_iter()
        .chain(streams.iter().map(|stream| &stream.tags))
        .find_map(|tags| {
            ["creation_time", "com.apple.quicktime.creationdate"]
                .into_iter()
                .find_map(|key| tag_value(tags, key).and_then(parse_capture_time))
        })
}

fn tag_value<'a>(tags: &'a HashMap<String, String>, key: &str) -> Option<&'a str> {
    tags.iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(key))
        .map(|(_, value)| value.as_str())
}

fn parse_capture_time(value: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(value.trim())
        .ok()
        .map(|datetime| datetime.timestamp_millis())
}

fn parse_frame_rate(rate: &String) -> Option<f64> {
    if let Some((num, den)) = rate.split_once('/') {
        let num: f64 = num.parse().ok()?;
        let den: f64 = den.parse().ok()?;
        if den == 0.0 {
            return None;
        }
        return Some(num / den);
    }
    rate.parse().ok()
}

fn find_ffprobe() -> AppResult<String> {
    // 优先检测 PATH
    if let Ok(found) = which::which("ffprobe") {
        return Ok(found.to_string_lossy().to_string());
    }
    if let Ok(found) = which::which("ffprobe.exe") {
        return Ok(found.to_string_lossy().to_string());
    }

    // 常见 Windows 安装位置
    let candidates = [
        r"C:\Program Files\FFmpeg\bin\ffprobe.exe",
        r"C:\Program Files (x86)\FFmpeg\bin\ffprobe.exe",
    ];
    for c in &candidates {
        if Path::new(c).exists() {
            return Ok(c.to_string());
        }
    }

    Err(AppError::FfprobeUnavailable(
        "未找到 ffprobe，请在设置中配置或安装 FFmpeg".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::{parse_capture_time, parse_frame_rate};

    #[test]
    fn parses_fractional_frame_rates() {
        assert_eq!(
            parse_frame_rate(&"30000/1001".to_string()),
            Some(30000.0 / 1001.0)
        );
        assert_eq!(parse_frame_rate(&"0/0".to_string()), None);
    }

    #[test]
    fn parses_rfc3339_capture_time_without_guessing_timezone() {
        assert_eq!(
            parse_capture_time("2024-05-06T07:08:09.123Z"),
            Some(1_714_979_289_123)
        );
        assert_eq!(parse_capture_time("2024-05-06 07:08:09"), None);
    }
}
