use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use exif::{In, Reader, Tag, Value};

/// Reads only timezone-qualified image timestamps. EXIF's traditional
/// DateTimeOriginal value has no timezone, so it is intentionally not treated
/// as an instant unless the matching OffsetTime field is also present.
pub fn probe_image_capture_time(path: &Path) -> Option<i64> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);
    let metadata = Reader::new().read_from_container(&mut reader).ok()?;

    [
        (Tag::DateTimeOriginal, Tag::OffsetTimeOriginal),
        (Tag::DateTimeDigitized, Tag::OffsetTimeDigitized),
        (Tag::DateTime, Tag::OffsetTime),
    ]
    .into_iter()
    .find_map(|(timestamp_tag, offset_tag)| {
        let timestamp = field_ascii(&metadata, timestamp_tag)?;
        let offset = field_ascii(&metadata, offset_tag)?;
        parse_exif_timestamp(&timestamp, &offset)
    })
}

fn field_ascii(metadata: &exif::Exif, tag: Tag) -> Option<String> {
    let field = metadata.get_field(tag, In::PRIMARY)?;
    let Value::Ascii(values) = &field.value else {
        return None;
    };
    let value = values.first()?;
    let value = std::str::from_utf8(value).ok()?.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn parse_exif_timestamp(timestamp: &str, offset: &str) -> Option<i64> {
    let value = format!("{timestamp}{offset}");
    [
        "%Y:%m:%d %H:%M:%S%.f%:z",
        "%Y:%m:%d %H:%M:%S%:z",
        "%Y:%m:%d %H:%M:%S%.f%z",
        "%Y:%m:%d %H:%M:%S%z",
    ]
    .into_iter()
    .find_map(|format| {
        chrono::DateTime::parse_from_str(&value, format)
            .ok()
            .map(|datetime| datetime.timestamp_millis())
    })
}

#[cfg(test)]
mod tests {
    use super::parse_exif_timestamp;

    #[test]
    fn parses_exif_original_time_with_explicit_offset() {
        assert_eq!(
            parse_exif_timestamp("2024:05:06 15:08:09", "+08:00"),
            Some(1_714_979_289_000)
        );
    }

    #[test]
    fn rejects_exif_time_without_offset() {
        assert_eq!(parse_exif_timestamp("2024:05:06 15:08:09", ""), None);
    }
}
