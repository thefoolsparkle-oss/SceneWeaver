use std::path::Path;

use image::{imageops, Rgb, RgbImage};

use crate::core::cache::CacheManager;
use crate::core::error::{AppError, AppResult};
use crate::models::{Asset, SelectItem};

pub fn write_csv(path: &Path, assets: &[Asset]) -> AppResult<()> {
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.eq_ignore_ascii_case("csv"))
        != Some(true)
    {
        return Err(AppError::Other("导出文件必须使用 .csv 扩展名".to_string()));
    }
    let mut output =
        String::from("file_name,file_path,media_type,duration_ms,start_timecode,end_timecode\r\n");
    for asset in assets {
        let duration = asset.duration_ms.unwrap_or(0);
        output.push_str(&format!(
            "{},{},{},{},{},{}\r\n",
            csv_escape(&asset.file_name),
            csv_escape(&asset.file_path),
            asset.media_type.as_str(),
            duration,
            timecode(0),
            timecode(duration),
        ));
    }
    std::fs::write(path, output)?;
    Ok(())
}

pub fn write_json(path: &Path, assets: &[Asset]) -> AppResult<()> {
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.eq_ignore_ascii_case("json"))
        != Some(true)
    {
        return Err(AppError::Other("导出文件必须使用 .json 扩展名".to_string()));
    }
    let items: Vec<serde_json::Value> = assets.iter().map(|asset| serde_json::json!({
        "file_name": asset.file_name, "file_path": asset.file_path, "media_type": asset.media_type.as_str(),
        "duration_ms": asset.duration_ms, "start_timecode": timecode(0), "end_timecode": timecode(asset.duration_ms.unwrap_or(0)),
    })).collect();
    std::fs::write(path, serde_json::to_string_pretty(&items)?)?;
    Ok(())
}

pub fn write_select_items_csv(path: &Path, items: &[SelectItem]) -> AppResult<()> {
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.eq_ignore_ascii_case("csv"))
        != Some(true)
    {
        return Err(AppError::Other("导出文件必须使用 .csv 扩展名".to_string()));
    }
    let mut output =
        String::from("file_name,file_path,media_type,start_timecode,end_timecode,rating,note\r\n");
    for item in items {
        let (start, end) = select_range(item);
        output.push_str(&format!(
            "{},{},{},{},{},{},{}\r\n",
            csv_escape(&item.asset.file_name),
            csv_escape(&item.asset.file_path),
            item.asset.media_type.as_str(),
            timecode(start),
            timecode(end),
            item.rating
                .map(|value| value.to_string())
                .unwrap_or_default(),
            csv_escape(item.note.as_deref().unwrap_or(""))
        ));
    }
    std::fs::write(path, output)?;
    Ok(())
}

pub fn write_select_contact_sheet_png(
    path: &Path,
    items: &[SelectItem],
    cache: &CacheManager,
) -> AppResult<()> {
    require_extension(path, "png")?;
    const TILE_WIDTH: u32 = 240;
    const TILE_HEIGHT: u32 = 150;
    const GAP: u32 = 8;
    const COLUMNS: u32 = 4;
    let rows = (items.len().max(1) as u32).div_ceil(COLUMNS);
    let width = COLUMNS * TILE_WIDTH + (COLUMNS + 1) * GAP;
    let height = rows * TILE_HEIGHT + (rows + 1) * GAP;
    let mut output = RgbImage::from_pixel(width, height, Rgb([24, 24, 27]));
    for (index, item) in items.iter().enumerate() {
        let source = item
            .segment
            .as_ref()
            .and_then(|segment| segment.thumbnail_path.as_ref())
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| cache.thumbnail_path(&item.asset.id, "cover"));
        let Ok(source_image) = image::open(source) else {
            continue;
        };
        let thumbnail = source_image.thumbnail(TILE_WIDTH, TILE_HEIGHT).to_rgb8();
        let column = index as u32 % COLUMNS;
        let row = index as u32 / COLUMNS;
        let x = GAP + column * (TILE_WIDTH + GAP) + (TILE_WIDTH - thumbnail.width()) / 2;
        let y = GAP + row * (TILE_HEIGHT + GAP) + (TILE_HEIGHT - thumbnail.height()) / 2;
        imageops::replace(&mut output, &thumbnail, x.into(), y.into());
    }
    output.save(path)?;
    Ok(())
}

pub fn write_select_items_json(path: &Path, items: &[SelectItem]) -> AppResult<()> {
    require_extension(path, "json")?;
    let values: Vec<serde_json::Value> = items
        .iter()
        .map(|item| {
            let (start, end) = select_range(item);
            serde_json::json!({
                "file_name": item.asset.file_name,
                "file_path": item.asset.file_path,
                "media_type": item.asset.media_type.as_str(),
                "start_ms": start,
                "end_ms": end,
                "start_timecode": timecode(start),
                "end_timecode": timecode(end),
                "rating": item.rating,
                "note": item.note,
            })
        })
        .collect();
    std::fs::write(path, serde_json::to_string_pretty(&values)?)?;
    Ok(())
}

pub fn write_select_items_edl(path: &Path, items: &[SelectItem]) -> AppResult<()> {
    require_extension(path, "edl")?;
    let mut output = String::from("TITLE: SceneWeaver Selects\r\nFCM: NON-DROP FRAME\r\n\r\n");
    let mut timeline_frame = 0_i64;
    for (index, item) in items.iter().enumerate() {
        let (start, end) = select_range(item);
        let source_in = frames_to_edl_timecode(milliseconds_to_frames(start));
        let duration_frames = milliseconds_to_frames(end - start).max(1);
        let source_out = frames_to_edl_timecode(milliseconds_to_frames(start) + duration_frames);
        let record_in = frames_to_edl_timecode(timeline_frame);
        let record_out = frames_to_edl_timecode(timeline_frame + duration_frames);
        output.push_str(&format!(
            "{:03}  AX       V     C        {source_in} {source_out} {record_in} {record_out}\r\n* FROM CLIP NAME: {}\r\n\r\n",
            index + 1,
            item.asset.file_name
        ));
        timeline_frame += duration_frames;
    }
    std::fs::write(path, output)?;
    Ok(())
}

pub fn write_select_items_fcpxml(path: &Path, items: &[SelectItem]) -> AppResult<()> {
    require_extension(path, "fcpxml")?;
    let total_ms: i64 = items
        .iter()
        .map(|item| {
            let (start, end) = select_range(item);
            (end - start).max(1)
        })
        .sum();
    let mut output = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<fcpxml version=\"1.10\"><resources><format id=\"r1\" frameDuration=\"1/30s\" width=\"1920\" height=\"1080\"/>\n");
    for (index, item) in items.iter().enumerate() {
        let (_, end) = select_range(item);
        let source_duration = item.asset.duration_ms.unwrap_or(end).max(end).max(1);
        output.push_str(&format!(
            "<asset id=\"r{}\" name=\"{}\" src=\"{}\" start=\"0s\" duration=\"{}\" {}/>\n",
            index + 1,
            xml_escape(&item.asset.file_name),
            file_url(&item.asset.file_path),
            fcpx_duration(source_duration),
            fcpx_media_attributes(&item.asset)
        ));
    }
    output.push_str(&format!("</resources><library><event name=\"SceneWeaver\"><project name=\"SceneWeaver Selects\"><sequence duration=\"{}\" format=\"r1\"><spine>\n", fcpx_duration(total_ms)));
    let mut offset = 0_i64;
    for (index, item) in items.iter().enumerate() {
        let (start, end) = select_range(item);
        let duration = (end - start).max(1);
        output.push_str(&format!(
            "<asset-clip ref=\"r{}\" name=\"{}\" offset=\"{}\" start=\"{}\" duration=\"{}\"/>\n",
            index + 1,
            xml_escape(&item.asset.file_name),
            fcpx_duration(offset),
            fcpx_duration(start),
            fcpx_duration(duration)
        ));
        offset += duration;
    }
    output.push_str("</spine></sequence></project></event></library></fcpxml>\n");
    std::fs::write(path, output)?;
    Ok(())
}

fn select_range(item: &SelectItem) -> (i64, i64) {
    let start = item
        .recommended_in_ms
        .or_else(|| item.segment.as_ref().map(|segment| segment.start_ms))
        .unwrap_or(0);
    let end = item
        .recommended_out_ms
        .or_else(|| item.segment.as_ref().map(|segment| segment.end_ms))
        .unwrap_or_else(|| item.asset.duration_ms.unwrap_or(0));
    (start, end.max(start))
}

fn require_extension(path: &Path, extension: &str) -> AppResult<()> {
    if path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.eq_ignore_ascii_case(extension))
        != Some(true)
    {
        return Err(AppError::Other(format!(
            "导出文件必须使用 .{extension} 扩展名"
        )));
    }
    Ok(())
}

pub fn write_edl(path: &Path, assets: &[Asset]) -> AppResult<()> {
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.eq_ignore_ascii_case("edl"))
        != Some(true)
    {
        return Err(AppError::Other("导出文件必须使用 .edl 扩展名".to_string()));
    }
    let mut output = String::from("TITLE: SceneWeaver Selects\r\nFCM: NON-DROP FRAME\r\n\r\n");
    let mut timeline_frame = 0_i64;
    for (index, asset) in assets.iter().enumerate() {
        let duration_frames = milliseconds_to_frames(asset.duration_ms.unwrap_or(0)).max(1);
        let source_out = frames_to_edl_timecode(duration_frames);
        let record_in = frames_to_edl_timecode(timeline_frame);
        let record_out = frames_to_edl_timecode(timeline_frame + duration_frames);
        output.push_str(&format!("{:03}  AX       V     C        00:00:00:00 {source_out} {record_in} {record_out}\r\n* FROM CLIP NAME: {}\r\n\r\n", index + 1, asset.file_name));
        timeline_frame += duration_frames;
    }
    std::fs::write(path, output)?;
    Ok(())
}

pub fn write_fcpxml(path: &Path, assets: &[Asset]) -> AppResult<()> {
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.eq_ignore_ascii_case("fcpxml"))
        != Some(true)
    {
        return Err(AppError::Other(
            "导出文件必须使用 .fcpxml 扩展名".to_string(),
        ));
    }
    let total_ms: i64 = assets
        .iter()
        .map(|asset| asset.duration_ms.unwrap_or(0).max(0))
        .sum();
    let mut output = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<fcpxml version=\"1.10\"><resources><format id=\"r1\" frameDuration=\"1/30s\" width=\"1920\" height=\"1080\"/>\n");
    for (index, asset) in assets.iter().enumerate() {
        output.push_str(&format!(
            "<asset id=\"r{}\" name=\"{}\" src=\"{}\" start=\"0s\" duration=\"{}\" {}/>\n",
            index + 1,
            xml_escape(&asset.file_name),
            file_url(&asset.file_path),
            fcpx_duration(asset.duration_ms.unwrap_or(0)),
            fcpx_media_attributes(asset)
        ));
    }
    output.push_str(&format!("</resources><library><event name=\"SceneWeaver\"><project name=\"SceneWeaver Selects\"><sequence duration=\"{}\" format=\"r1\"><spine>\n", fcpx_duration(total_ms)));
    let mut offset = 0_i64;
    for (index, asset) in assets.iter().enumerate() {
        let duration = asset.duration_ms.unwrap_or(0).max(0);
        output.push_str(&format!(
            "<asset-clip ref=\"r{}\" name=\"{}\" offset=\"{}\" start=\"0s\" duration=\"{}\"/>\n",
            index + 1,
            xml_escape(&asset.file_name),
            fcpx_duration(offset),
            fcpx_duration(duration)
        ));
        offset += duration;
    }
    output.push_str("</spine></sequence></project></event></library></fcpxml>\n");
    std::fs::write(path, output)?;
    Ok(())
}

fn fcpx_duration(milliseconds: i64) -> String {
    format!("{}/1000s", milliseconds.max(0))
}
fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
fn file_url(path: &str) -> String {
    let mut encoded = String::new();
    for byte in path.replace('\\', "/").bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'/' | b':') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    format!("file:///{encoded}")
}

fn fcpx_media_attributes(asset: &Asset) -> &'static str {
    match asset.media_type.as_str() {
        "audio" => "hasAudio=\"1\"",
        "image" => "hasVideo=\"1\"",
        _ => "hasVideo=\"1\" hasAudio=\"1\"",
    }
}

fn milliseconds_to_frames(milliseconds: i64) -> i64 {
    ((milliseconds.max(0) as f64 / 1000.0) * 30.0).round() as i64
}
pub fn frames_to_edl_timecode(frames: i64) -> String {
    let frames = frames.max(0);
    let total_seconds = frames / 30;
    format!(
        "{:02}:{:02}:{:02}:{:02}",
        total_seconds / 3600,
        (total_seconds / 60) % 60,
        total_seconds % 60,
        frames % 30
    )
}

fn csv_escape(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

pub fn timecode(milliseconds: i64) -> String {
    let total_seconds = milliseconds.max(0) / 1000;
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        total_seconds / 3600,
        (total_seconds / 60) % 60,
        total_seconds % 60,
        milliseconds.max(0) % 1000
    )
}

#[cfg(test)]
mod tests {
    use super::{csv_escape, file_url, timecode};

    #[test]
    fn formats_milliseconds_as_timecode() {
        assert_eq!(timecode(3_661_234), "01:01:01.234");
    }

    #[test]
    fn escapes_csv_quotes() {
        assert_eq!(csv_escape("a,\"b\""), "\"a,\"\"b\"\"\"");
    }

    #[test]
    fn formats_edl_timecode_at_thirty_fps() {
        assert_eq!(super::frames_to_edl_timecode(30 * 61 + 12), "00:01:01:12");
    }

    #[test]
    fn escapes_fcpxml_values() {
        assert_eq!(super::xml_escape("A&B"), "A&amp;B");
    }

    #[test]
    fn encodes_non_ascii_and_reserved_file_url_characters() {
        assert_eq!(
            file_url(r"C:\素材 库\雨夜#1%.mp4"),
            "file:///C:/%E7%B4%A0%E6%9D%90%20%E5%BA%93/%E9%9B%A8%E5%A4%9C%231%25.mp4"
        );
    }
}
