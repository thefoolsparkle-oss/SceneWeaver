use std::path::{Path, PathBuf};

use crate::core::error::AppResult;

#[derive(Clone)]
pub struct CacheManager {
    root: PathBuf,
}

impl CacheManager {
    pub fn new(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref().to_path_buf();
        Self { root }
    }

    pub fn ensure_dirs(&self) -> AppResult<()> {
        for sub in ["thumbnails", "proxies", "keyframes", "waveforms", "models"] {
            std::fs::create_dir_all(self.root.join(sub))?;
        }
        Ok(())
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn thumbnail_path(&self, asset_id: &str, variant: &str) -> PathBuf {
        self.root
            .join("thumbnails")
            .join(format!("{}_{}.jpg", asset_id, variant))
    }

    pub fn proxy_path(&self, asset_id: &str) -> PathBuf {
        self.root.join("proxies").join(format!("{}.mp4", asset_id))
    }

    pub fn segment_preview_path(&self, asset_id: &str, index: i32) -> PathBuf {
        self.root
            .join("proxies")
            .join(format!("{}_{}.mp4", asset_id, index))
    }

    pub fn keyframe_path(&self, asset_id: &str, index: i32) -> PathBuf {
        self.root
            .join("keyframes")
            .join(format!("{}_{}.jpg", asset_id, index))
    }

    pub fn models_path(&self) -> PathBuf {
        self.root.join("models")
    }

    /// Removes media derivatives tied to one asset while leaving the model
    /// cache and derivatives belonging to every other asset untouched.
    ///
    /// Asset ids are generated UUIDs, so the prefix cannot overlap with a
    /// different asset id. This is used before a changed source file is
    /// re-indexed; otherwise an old keyframe or short preview could be shown
    /// for the new bytes at the same filesystem path.
    pub fn remove_asset_derivatives(&self, asset_id: &str) -> AppResult<u64> {
        let mut removed_bytes = 0u64;
        for (subdirectory, prefix) in [
            ("thumbnails", format!("{asset_id}_")),
            ("proxies", format!("{asset_id}_")),
            ("keyframes", format!("{asset_id}_")),
        ] {
            let directory = self.root.join(subdirectory);
            if !directory.is_dir() {
                continue;
            }
            for entry in std::fs::read_dir(directory)? {
                let entry = entry?;
                if !entry.file_type()?.is_file() {
                    continue;
                }
                let Some(file_name) = entry.file_name().to_str().map(str::to_string) else {
                    continue;
                };
                if file_name.starts_with(&prefix) {
                    removed_bytes += entry.metadata()?.len();
                    std::fs::remove_file(entry.path())?;
                }
            }
        }
        Ok(removed_bytes)
    }

    pub fn clear_cache(&self) -> AppResult<u64> {
        let mut total = 0u64;
        for sub in ["thumbnails", "proxies", "keyframes", "waveforms"] {
            let dir = self.root.join(sub);
            if dir.exists() {
                for entry in std::fs::read_dir(&dir)? {
                    let entry = entry?;
                    let meta = entry.metadata()?;
                    total += meta.len();
                    if meta.is_file() {
                        std::fs::remove_file(entry.path())?;
                    }
                }
            }
        }
        Ok(total)
    }

    pub fn cache_size(&self) -> AppResult<u64> {
        let mut total = 0u64;
        for sub in ["thumbnails", "proxies", "keyframes", "waveforms"] {
            let dir = self.root.join(sub);
            if dir.exists() {
                for entry in std::fs::read_dir(&dir)? {
                    let entry = entry?;
                    if entry.metadata()?.is_file() {
                        total += entry.metadata()?.len();
                    }
                }
            }
        }
        Ok(total)
    }
}
