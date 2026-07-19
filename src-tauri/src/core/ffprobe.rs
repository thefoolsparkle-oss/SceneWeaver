use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

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
}

pub fn probe_media(path: &Path) -> AppResult<MediaProbeInfo> {
    let ffprobe_exe = find_ffprobe()?;

    let source = path.to_path_buf();
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let result = Command::new(ffprobe_exe)
            .args([
                "-v",
                "error",
                "-print_format",
                "json",
                "-show_format",
                "-show_streams",
            ])
            .arg(source)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();
        let _ = sender.send(result);
    });
    let output = receiver
        .recv_timeout(Duration::from_secs(30))
        .map_err(|_| AppError::FfprobeUnavailable("ffprobe 超时（30 秒）".to_string()))??;

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
    })
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
    use super::parse_frame_rate;

    #[test]
    fn parses_fractional_frame_rates() {
        assert_eq!(
            parse_frame_rate(&"30000/1001".to_string()),
            Some(30000.0 / 1001.0)
        );
        assert_eq!(parse_frame_rate(&"0/0".to_string()), None);
    }
}
