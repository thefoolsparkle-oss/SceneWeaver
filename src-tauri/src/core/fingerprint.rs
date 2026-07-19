use std::fs;
use std::path::Path;

use blake3::Hasher;

use crate::core::error::AppResult;

/// 快速指纹：基于路径、大小、修改时间，足够判断日常文件变化。
pub fn quick_fingerprint(normalized_path: &str, size: u64, modified_ms: i64) -> String {
    format!("{}|{}|{}", normalized_path, size, modified_ms)
}

/// 完整哈希：基于文件内容，用于重命名/移动检测。
pub fn full_hash(path: &Path) -> AppResult<String> {
    let mut hasher = Hasher::new();
    let mut file = fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(&mut file);
    std::io::copy(&mut reader, &mut hasher)?;
    Ok(hasher.finalize().to_hex().to_string())
}

/// 判断是否需要重新分析：指纹不同或资产状态为 error/offline。
pub fn needs_reindex(existing_fingerprint: &str, new_fingerprint: &str, status: &str) -> bool {
    existing_fingerprint != new_fingerprint || status == "error" || status == "offline"
}

#[cfg(test)]
mod tests {
    use super::{needs_reindex, quick_fingerprint};

    #[test]
    fn fingerprint_is_stable_for_a_chinese_path() {
        let path = r"E:\素材 库\夜景\镜头 01.mp4";
        assert_eq!(
            quick_fingerprint(path, 42, 1000),
            quick_fingerprint(path, 42, 1000)
        );
    }

    #[test]
    fn unchanged_indexed_asset_does_not_reindex() {
        assert!(!needs_reindex("same", "same", "indexed"));
        assert!(needs_reindex("same", "new", "indexed"));
        assert!(needs_reindex("same", "same", "offline"));
    }
}
