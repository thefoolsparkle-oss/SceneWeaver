use std::path::Path;

use crate::core::error::AppResult;

pub const PROVIDER_ID: &str = "local-color-histogram";
pub const MODEL_VERSION: &str = "v1";
const BINS_PER_CHANNEL: usize = 8;

/// A deterministic, offline image feature provider. It is intentionally small so
/// reference-image retrieval remains available without a downloaded ML model.
/// A CLIP-compatible provider can replace it without changing the persistence API.
pub fn embed_image(path: &Path) -> AppResult<Vec<f32>> {
    let image = image::open(path)?.to_rgb8();
    let mut vector = vec![0.0_f32; BINS_PER_CHANNEL * 3];
    let pixel_count = (image.width() as usize * image.height() as usize).max(1) as f32;
    for pixel in image.pixels() {
        for channel in 0..3 {
            let bin = (pixel[channel] as usize * BINS_PER_CHANNEL) / 256;
            vector[channel * BINS_PER_CHANNEL + bin.min(BINS_PER_CHANNEL - 1)] += 1.0;
        }
    }
    for value in &mut vector {
        *value /= pixel_count;
    }
    Ok(vector)
}

pub fn cosine_similarity(left: &[f32], right: &[f32]) -> f32 {
    if left.len() != right.len() || left.is_empty() {
        return 0.0;
    }
    let (mut dot, mut left_norm, mut right_norm) = (0.0, 0.0, 0.0);
    for (&a, &b) in left.iter().zip(right) {
        dot += a * b;
        left_norm += a * a;
        right_norm += b * b;
    }
    dot / (left_norm.sqrt() * right_norm.sqrt()).max(f32::EPSILON)
}

/// Maps a small, transparent subset of Chinese/English colour language onto the
/// same feature space as the offline image provider. Unknown text deliberately
/// returns None so callers can keep using keyword search instead of pretending
/// to understand an arbitrary visual description.
pub fn embed_color_query(query: &str) -> Option<Vec<f32>> {
    let normalized = query.to_lowercase();
    let channel = if normalized.contains("红") || normalized.contains("red") {
        0
    } else if normalized.contains("绿") || normalized.contains("green") {
        1
    } else if normalized.contains("蓝") || normalized.contains("blue") || normalized.contains("夜")
    {
        2
    } else if normalized.contains("黄") || normalized.contains("yellow") {
        return Some(colour_vector(&[(0, 1.0), (1, 1.0)]));
    } else {
        return None;
    };
    Some(colour_vector(&[(channel, 1.0)]))
}

fn colour_vector(channels: &[(usize, f32)]) -> Vec<f32> {
    let mut vector = vec![0.0; BINS_PER_CHANNEL * 3];
    for (channel, weight) in channels {
        vector[channel * BINS_PER_CHANNEL + BINS_PER_CHANNEL - 1] = *weight;
    }
    vector
}

#[cfg(test)]
mod tests {
    use super::{cosine_similarity, embed_color_query};

    #[test]
    fn ranks_matching_features_higher_than_unrelated_ones() {
        let red = [1.0, 0.0, 0.0];
        let near_red = [0.9, 0.1, 0.0];
        let blue = [0.0, 0.0, 1.0];
        assert!(cosine_similarity(&red, &near_red) > cosine_similarity(&red, &blue));
    }

    #[test]
    fn maps_chinese_colour_language_without_claiming_general_semantics() {
        assert!(embed_color_query("红色跑车").is_some());
        assert!(embed_color_query("蓝色夜景").is_some());
        assert!(embed_color_query("人物回头").is_none());
    }
}
