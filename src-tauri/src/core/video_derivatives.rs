use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use chrono::Utc;

use crate::core::cache::CacheManager;
use crate::core::error::{AppError, AppResult};
use crate::models::Segment;

const MAX_PREVIEW_DURATION_MS: i64 = 6_000;
const MAX_DERIVATIVE_SEGMENTS: usize = 120;
const PROCESS_TIMEOUT: Duration = Duration::from_secs(90);

/// Generates a representative JPEG and a small playable MP4 per detected segment.
///
/// A missing FFmpeg is an expected optional-dependency case: the caller still gets
/// persisted shot boundaries, just without preview media. Other FFmpeg failures are
/// returned so the UI can explain that the original media is still intact.
pub fn generate_for_segments(
    cache: &CacheManager,
    source: &Path,
    asset_id: &str,
    segments: &mut [Segment],
) -> AppResult<bool> {
    let Ok(ffmpeg) = find_ffmpeg() else {
        return Ok(false);
    };
    for segment in segments.iter_mut().take(MAX_DERIVATIVE_SEGMENTS) {
        let keyframe = cache.keyframe_path(asset_id, segment.segment_index);
        let preview = cache.segment_preview_path(asset_id, segment.segment_index);
        let timestamp_ms = representative_timestamp_ms(segment.start_ms, segment.duration_ms);

        run_ffmpeg(&ffmpeg, source, &keyframe_args(timestamp_ms, &keyframe))?;
        run_ffmpeg(
            &ffmpeg,
            source,
            &preview_args(segment.start_ms, segment.duration_ms, &preview),
        )?;

        let (black_frame_score, blur_score, quality_score) = quality_metrics(&keyframe)?;
        let now = Utc::now().timestamp_millis();
        segment.representative_frame_path = Some(keyframe.to_string_lossy().to_string());
        segment.thumbnail_path = Some(keyframe.to_string_lossy().to_string());
        segment.preview_path = Some(preview.to_string_lossy().to_string());
        segment.black_frame_score = Some(black_frame_score);
        segment.blur_score = Some(blur_score);
        segment.quality_score = Some(quality_score);
        segment.updated_at = now;
    }
    Ok(true)
}

fn find_ffmpeg() -> AppResult<PathBuf> {
    which::which("ffmpeg")
        .or_else(|_| which::which("ffmpeg.exe"))
        .map_err(|_| {
            AppError::FfprobeUnavailable("未找到 FFmpeg，无法生成视频关键帧和预览".to_string())
        })
}

fn run_ffmpeg(executable: &Path, source: &Path, args: &[String]) -> AppResult<()> {
    if let Some(parent) = args.last().and_then(|output| Path::new(output).parent()) {
        std::fs::create_dir_all(parent)?;
    }
    let executable = executable.to_path_buf();
    let source = source.to_path_buf();
    let args = args.to_vec();
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let result = Command::new(executable)
            .arg("-hide_banner")
            .arg("-y")
            .arg("-i")
            .arg(source)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output();
        let _ = sender.send(result);
    });
    let output = receiver
        .recv_timeout(PROCESS_TIMEOUT)
        .map_err(|_| AppError::Other("视频派生文件生成超时（90 秒）".to_string()))??;
    if output.status.success() {
        Ok(())
    } else {
        let details = String::from_utf8_lossy(&output.stderr);
        let message = details.lines().last().unwrap_or("未知 FFmpeg 错误");
        Err(AppError::Other(format!(
            "FFmpeg 生成视频预览失败: {message}"
        )))
    }
}

fn representative_timestamp_ms(start_ms: i64, duration_ms: i64) -> i64 {
    start_ms.saturating_add((duration_ms.max(1) / 2).min(1_000))
}

fn preview_duration_ms(duration_ms: i64) -> i64 {
    duration_ms.clamp(250, MAX_PREVIEW_DURATION_MS)
}

fn keyframe_args(timestamp_ms: i64, output: &Path) -> Vec<String> {
    vec![
        "-ss".into(),
        format_seconds(timestamp_ms),
        "-frames:v".into(),
        "1".into(),
        "-vf".into(),
        "scale='min(640,iw)':-2".into(),
        "-q:v".into(),
        "3".into(),
        output.to_string_lossy().to_string(),
    ]
}

fn preview_args(start_ms: i64, duration_ms: i64, output: &Path) -> Vec<String> {
    vec![
        "-ss".into(),
        format_seconds(start_ms),
        "-t".into(),
        format_seconds(preview_duration_ms(duration_ms)),
        "-an".into(),
        "-vf".into(),
        "scale='min(640,iw)':-2,fps=15".into(),
        "-c:v".into(),
        "libx264".into(),
        "-preset".into(),
        "veryfast".into(),
        "-crf".into(),
        "32".into(),
        "-movflags".into(),
        "+faststart".into(),
        output.to_string_lossy().to_string(),
    ]
}

fn format_seconds(milliseconds: i64) -> String {
    format!("{:.3}", milliseconds.max(0) as f64 / 1000.0)
}

fn quality_metrics(path: &Path) -> AppResult<(f64, f64, f64)> {
    let image = image::open(path)?.to_luma8();
    let pixels = image.as_raw();
    if pixels.is_empty() {
        return Ok((1.0, 1.0, 0.0));
    }
    let black = pixels.iter().filter(|&&pixel| pixel <= 16).count() as f64 / pixels.len() as f64;
    let width = image.width() as usize;
    let height = image.height() as usize;
    let mut edges = 0.0;
    let mut comparisons = 0usize;
    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;
            if x + 1 < width {
                edges += (pixels[index] as f64 - pixels[index + 1] as f64).abs();
                comparisons += 1;
            }
            if y + 1 < height {
                edges += (pixels[index] as f64 - pixels[index + width] as f64).abs();
                comparisons += 1;
            }
        }
    }
    let detail = (edges / comparisons.max(1) as f64 / 28.0).clamp(0.0, 1.0);
    let blur = 1.0 - detail;
    let quality = ((1.0 - black) * 0.6 + detail * 0.4).clamp(0.0, 1.0);
    Ok((black, blur, quality))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{keyframe_args, preview_args, preview_duration_ms, representative_timestamp_ms};

    #[test]
    fn caps_preview_and_keeps_short_shots_playable() {
        assert_eq!(preview_duration_ms(10_000), 6_000);
        assert_eq!(preview_duration_ms(100), 250);
    }

    #[test]
    fn chooses_a_frame_inside_the_segment() {
        assert_eq!(representative_timestamp_ms(4_000, 8_000), 5_000);
        assert_eq!(representative_timestamp_ms(4_000, 300), 4_150);
    }

    #[test]
    fn ffmpeg_arguments_never_require_a_shell() {
        let keyframe = keyframe_args(1_250, Path::new("C:/cache/中文 frame.jpg"));
        let preview = preview_args(2_000, 10_000, Path::new("C:/cache/中文 preview.mp4"));
        assert_eq!(keyframe[0..2], ["-ss", "1.250"]);
        assert!(preview.windows(2).any(|pair| pair == ["-t", "6.000"]));
        assert_eq!(
            preview.last().map(String::as_str),
            Some("C:/cache/中文 preview.mp4")
        );
    }
}
