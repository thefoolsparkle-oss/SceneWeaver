use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use image::{imageops::FilterType, DynamicImage, ImageReader};

use crate::core::cache::CacheManager;
use crate::core::error::{AppError, AppResult};
use crate::models::{Asset, MediaType};

pub struct ThumbnailService {
    cache: CacheManager,
}

impl ThumbnailService {
    pub fn new(cache: CacheManager) -> Self {
        Self { cache }
    }

    pub fn remove_derivatives_for_asset(&self, asset_id: &str) -> AppResult<u64> {
        self.cache.remove_asset_derivatives(asset_id)
    }

    pub fn generate_for_asset(&self, asset: &Asset) -> AppResult<Option<PathBuf>> {
        let output_path = self.cache.thumbnail_path(&asset.id, "cover");
        if output_path.exists() {
            return Ok(Some(output_path));
        }

        match asset.media_type {
            MediaType::Image => generate_image_thumbnail(&asset.file_path, &output_path),
            MediaType::Video => {
                generate_video_thumbnail(&asset.file_path, &output_path, asset.duration_ms)
            }
            MediaType::Audio => Ok(None),
        }
    }
}

fn generate_image_thumbnail(input: &str, output: &Path) -> AppResult<Option<PathBuf>> {
    let img = ImageReader::open(input)?;
    let img = img
        .decode()
        .map_err(|e| AppError::Other(format!("图片解码失败: {}", e)))?;
    let thumb = resize_to_cover(&img, 320, 180);
    save_jpeg(&thumb, output)?;
    Ok(Some(output.to_path_buf()))
}

fn generate_video_thumbnail(
    input: &str,
    output: &Path,
    duration_ms: Option<i64>,
) -> AppResult<Option<PathBuf>> {
    let ffmpeg = match find_ffmpeg() {
        Ok(p) => p,
        Err(_) => return Ok(None),
    };

    let time = duration_ms.map(|d| d as f64 / 2000.0).unwrap_or(1.0);
    let time_str = format!("{:.3}", time.max(0.1));

    let input = input.to_string();
    let output = output.to_path_buf();
    let command_output = output.clone();
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let result = Command::new(ffmpeg)
            .args([
                "-ss",
                &time_str,
                "-i",
                &input,
                "-vf",
                "scale=320:180:force_original_aspect_ratio=decrease,pad=320:180:(ow-iw)/2:(oh-ih)/2",
                "-frames:v",
                "1",
                "-q:v",
                "2",
                "-y",
            ])
            .arg(&command_output)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success());
        let _ = sender.send(result);
    });
    let result = receiver
        .recv_timeout(Duration::from_secs(30))
        .ok()
        .and_then(Result::ok);

    match result {
        Some(true) => Ok(Some(output)),
        _ => Ok(None),
    }
}

fn resize_to_cover(img: &DynamicImage, width: u32, height: u32) -> DynamicImage {
    let mut resized = img.resize(width, height, FilterType::Lanczos3);
    let (rw, rh) = (resized.width(), resized.height());
    if rw == width && rh == height {
        return resized;
    }
    // 如果比例不完全匹配，再做中心裁剪
    let cropped = resized.crop(
        (rw.saturating_sub(width)) / 2,
        (rh.saturating_sub(height)) / 2,
        width.min(rw),
        height.min(rh),
    );
    DynamicImage::from(cropped)
}

fn save_jpeg(img: &DynamicImage, output: &Path) -> AppResult<()> {
    let rgb = img.to_rgb8();
    rgb.save_with_format(output, image::ImageFormat::Jpeg)
        .map_err(|e| AppError::Other(format!("保存缩略图失败: {}", e)))?;
    Ok(())
}

fn find_ffmpeg() -> AppResult<String> {
    if let Ok(found) = which::which("ffmpeg") {
        return Ok(found.to_string_lossy().to_string());
    }
    if let Ok(found) = which::which("ffmpeg.exe") {
        return Ok(found.to_string_lossy().to_string());
    }
    let candidates = [
        r"C:\Program Files\FFmpeg\bin\ffmpeg.exe",
        r"C:\Program Files (x86)\FFmpeg\bin\ffmpeg.exe",
    ];
    for c in &candidates {
        if Path::new(c).exists() {
            return Ok(c.to_string());
        }
    }
    Err(AppError::FfprobeUnavailable("未找到 ffmpeg".to_string()))
}
