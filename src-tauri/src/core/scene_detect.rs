use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use chrono::Utc;

use crate::core::error::{AppError, AppResult};
use crate::models::Segment;

const SCENE_THRESHOLD: &str = "0.30";
const MIN_SHOT_DURATION_MS: i64 = 400;
const MAX_BOUNDARIES: usize = 500;

pub fn detect_shots(asset_id: &str, path: &Path, duration_ms: i64) -> AppResult<Vec<Segment>> {
    let boundaries = match find_ffmpeg() {
        Ok(ffmpeg) => run_scene_detection(&ffmpeg, path)?,
        Err(_) => Vec::new(),
    };
    Ok(build_segments(asset_id, duration_ms, &boundaries))
}

fn run_scene_detection(ffmpeg: &str, path: &Path) -> AppResult<Vec<i64>> {
    let executable = ffmpeg.to_string();
    let source = path.to_path_buf();
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let filter = format!("select='gt(scene,{SCENE_THRESHOLD})',showinfo");
        let result = Command::new(executable)
            .args(["-hide_banner", "-i"])
            .arg(source)
            .args(["-an", "-vf", &filter, "-f", "null", "-"])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output();
        let _ = sender.send(result);
    });
    let output = receiver
        .recv_timeout(Duration::from_secs(90))
        .map_err(|_| AppError::Other("镜头检测超时（90 秒）".to_string()))??;
    if !output.status.success() {
        return Err(AppError::Other("FFmpeg 镜头检测失败".to_string()));
    }
    Ok(parse_scene_timestamps(&String::from_utf8_lossy(
        &output.stderr,
    )))
}

fn parse_scene_timestamps(output: &str) -> Vec<i64> {
    output
        .lines()
        .filter_map(|line| line.split("pts_time:").nth(1))
        .filter_map(|tail| tail.split_whitespace().next())
        .filter_map(|value| value.parse::<f64>().ok())
        .map(|seconds| (seconds * 1000.0).round() as i64)
        .take(MAX_BOUNDARIES)
        .collect()
}

pub fn build_segments(asset_id: &str, duration_ms: i64, boundaries: &[i64]) -> Vec<Segment> {
    let duration_ms = duration_ms.max(1);
    let mut starts = vec![0];
    for &boundary in boundaries {
        if boundary > *starts.last().unwrap_or(&0) + MIN_SHOT_DURATION_MS && boundary < duration_ms
        {
            starts.push(boundary);
        }
    }
    let now = Utc::now().timestamp_millis();
    starts
        .iter()
        .enumerate()
        .map(|(index, start)| {
            let end = starts.get(index + 1).copied().unwrap_or(duration_ms);
            Segment {
                id: uuid::Uuid::new_v4().to_string(),
                asset_id: asset_id.to_string(),
                segment_type: if boundaries.is_empty() {
                    "whole_asset"
                } else {
                    "shot"
                }
                .to_string(),
                segment_index: index as i32,
                start_ms: *start,
                end_ms: end,
                duration_ms: end - start,
                representative_frame_path: None,
                thumbnail_path: None,
                thumbnail_data_url: None,
                preview_path: None,
                quality_score: None,
                subtitle_present: None,
                game_ui: None,
                black_frame_score: None,
                blur_score: None,
                embedding_ref: None,
                created_at: now,
                updated_at: now,
            }
        })
        .collect()
}

fn find_ffmpeg() -> AppResult<String> {
    which::which("ffmpeg")
        .or_else(|_| which::which("ffmpeg.exe"))
        .map(|path| path.to_string_lossy().to_string())
        .map_err(|_| {
            AppError::FfprobeUnavailable("未找到 FFmpeg，视频将保留为完整片段".to_string())
        })
}

#[cfg(test)]
mod tests {
    use super::{build_segments, parse_scene_timestamps};

    #[test]
    fn parses_ffmpeg_scene_timestamps() {
        assert_eq!(
            parse_scene_timestamps("[Parsed_showinfo] pts_time:1.250 foo\npts_time:3.0"),
            vec![1250, 3000]
        );
    }

    #[test]
    fn removes_too_close_boundaries_and_closes_final_segment() {
        let segments = build_segments("asset", 5000, &[100, 1000, 1200, 4000]);
        assert_eq!(segments.len(), 3);
        assert_eq!((segments[0].start_ms, segments[0].end_ms), (0, 1000));
        assert_eq!((segments[2].start_ms, segments[2].end_ms), (4000, 5000));
    }
}
