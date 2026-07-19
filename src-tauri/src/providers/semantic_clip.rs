//! Optional local CLIP provider.
//!
//! The model is downloaded only by the explicit Settings action.  A missing
//! model or ONNX Runtime is a normal, non-fatal state: callers must keep the
//! keyword and colour-index paths available.
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::Serialize;

use crate::core::error::{AppError, AppResult};

pub const PROVIDER_ID: &str = "local-clip-vit-b32";
pub const MODEL_VERSION: &str = "qdrant-clip-vit-b32-v1";
const READY_MARKER: &str = "semantic-clip-v1.ready";
static ORT_INITIALIZED: OnceLock<Result<(), String>> = OnceLock::new();

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticModelStatus {
    pub ready: bool,
    pub model_installed: bool,
    pub runtime_available: bool,
    pub provider_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticIndexResult {
    pub indexed: usize,
    pub skipped: usize,
    pub failed: usize,
    pub entity_references_indexed: usize,
    pub entity_references_skipped: usize,
    pub entity_references_failed: usize,
}

pub fn status(cache_models: &Path, runtime_path: &Path) -> SemanticModelStatus {
    let model_installed =
        cache_models.join(READY_MARKER).is_file() && required_model_files_exist(cache_models);
    let runtime_available = runtime_path.is_file();
    let message = if !runtime_available {
        "未检测到随应用分发的 ONNX Runtime；关键词和颜色检索仍可用".to_string()
    } else if !model_installed {
        "模型未完整安装或文件已被移除；点击下载后才会访问网络，核心本地流程不受影响".to_string()
    } else {
        "本地 CLIP 模型已就绪；可重新建立语义向量索引".to_string()
    };
    SemanticModelStatus {
        ready: model_installed && runtime_available,
        model_installed,
        runtime_available,
        provider_id: PROVIDER_ID.to_string(),
        message,
    }
}

/// Downloads the paired CLIP encoders into the application's model cache.
/// This function is only reached through the user-initiated command.
pub fn install(cache_models: &Path, runtime_path: &Path) -> AppResult<SemanticModelStatus> {
    std::fs::create_dir_all(cache_models)?;
    ensure_runtime(runtime_path)?;
    let options = fastembed::ImageInitOptions::new(fastembed::ImageEmbeddingModel::ClipVitB32)
        .with_cache_dir(cache_models.to_path_buf())
        .with_show_download_progress(false);
    fastembed::ImageEmbedding::try_new(options)
        .map_err(|error| AppError::Other(format!("下载 CLIP 图像模型失败: {error}")))?;
    let options = fastembed::TextInitOptions::new(fastembed::EmbeddingModel::ClipVitB32)
        .with_cache_dir(cache_models.to_path_buf())
        .with_show_download_progress(false);
    fastembed::TextEmbedding::try_new(options)
        .map_err(|error| AppError::Other(format!("下载 CLIP 文本模型失败: {error}")))?;
    std::fs::write(cache_models.join(READY_MARKER), MODEL_VERSION)?;
    Ok(status(cache_models, runtime_path))
}

pub fn embed_image(cache_models: &Path, runtime_path: &Path, path: &Path) -> AppResult<Vec<f32>> {
    require_ready(cache_models, runtime_path)?;
    let options = fastembed::ImageInitOptions::new(fastembed::ImageEmbeddingModel::ClipVitB32)
        .with_cache_dir(cache_models.to_path_buf())
        .with_show_download_progress(false);
    let mut model = fastembed::ImageEmbedding::try_new(options)
        .map_err(|error| AppError::Other(format!("载入本地 CLIP 图像模型失败: {error}")))?;
    model
        .embed(vec![path.to_path_buf()], None)
        .map_err(|error| AppError::Other(format!("CLIP 图像向量计算失败: {error}")))?
        .into_iter()
        .next()
        .ok_or_else(|| AppError::Other("CLIP 未返回图像向量".to_string()))
}

pub fn embed_text(cache_models: &Path, runtime_path: &Path, query: &str) -> AppResult<Vec<f32>> {
    require_ready(cache_models, runtime_path)?;
    let options = fastembed::TextInitOptions::new(fastembed::EmbeddingModel::ClipVitB32)
        .with_cache_dir(cache_models.to_path_buf())
        .with_show_download_progress(false);
    let mut model = fastembed::TextEmbedding::try_new(options)
        .map_err(|error| AppError::Other(format!("载入本地 CLIP 文本模型失败: {error}")))?;
    model
        .embed(vec![query.to_string()], None)
        .map_err(|error| AppError::Other(format!("CLIP 文本向量计算失败: {error}")))?
        .into_iter()
        .next()
        .ok_or_else(|| AppError::Other("CLIP 未返回文本向量".to_string()))
}

/// CLIP ViT-B/32's paired text encoder is English-oriented.  This deliberately
/// small local glossary gives common creator and ACG Chinese prompts a useful
/// English representation without claiming arbitrary Chinese understanding.
/// Unknown Han-only input returns `None` so callers retain the honest keyword
/// fallback instead of producing an opaque low-quality vector result.
pub fn semantic_query_prompt(query: &str) -> Option<String> {
    let normalized = query.trim();
    if normalized.is_empty() {
        return None;
    }
    let glossary = [
        ("雨夜", "rainy night"),
        ("下雨", "rain"),
        ("雨天", "rainy day"),
        ("雨伞", "umbrella"),
        ("夜景", "night scene"),
        ("夜晚", "night"),
        ("白天", "daytime"),
        ("日落", "sunset"),
        ("黄昏", "dusk"),
        ("清晨", "dawn"),
        ("角色", "character"),
        ("人物", "person"),
        ("女孩", "girl"),
        ("男孩", "boy"),
        ("人群", "crowd"),
        ("侧脸", "side profile"),
        ("正脸", "front face"),
        ("背影", "back view"),
        ("回头", "looking back"),
        ("近景", "close up"),
        ("特写", "close up"),
        ("中景", "medium shot"),
        ("远景", "wide shot"),
        ("俯拍", "high angle"),
        ("仰拍", "low angle"),
        ("第一人称", "first person view"),
        ("城市", "city"),
        ("街道", "street"),
        ("室内", "indoors"),
        ("室外", "outdoors"),
        ("学校", "school"),
        ("海边", "beach"),
        ("战斗", "battle"),
        ("打斗", "fight"),
        ("奔跑", "running"),
        ("走路", "walking"),
        ("跳跃", "jumping"),
        ("拥抱", "hugging"),
        ("游戏", "video game"),
        ("界面", "user interface"),
        ("字幕", "subtitles"),
        ("微笑", "smiling"),
        ("大笑", "laughing"),
        ("哭", "crying"),
        ("愤怒", "angry"),
        ("惊讶", "surprised"),
        ("悲伤", "sad"),
        ("害羞", "shy"),
        ("粉色头发", "pink hair"),
        ("黑色头发", "black hair"),
        ("白色头发", "white hair"),
        ("金色头发", "blonde hair"),
        ("红", "red"),
        ("蓝", "blue"),
        ("绿色", "green"),
        ("黄色", "yellow"),
    ];
    let mut phrases = glossary
        .iter()
        .filter(|(chinese, _)| normalized.contains(chinese))
        .map(|(_, english)| *english)
        .collect::<Vec<_>>();
    phrases.sort_unstable();
    phrases.dedup();
    if phrases.is_empty() {
        if normalized.chars().any(is_han) {
            None
        } else {
            Some(normalized.to_string())
        }
    } else {
        Some(phrases.join(", "))
    }
}

/// Builds the CLIP prompt for a parsed creator search. Explicit exclusions
/// are deliberately absent: including `不要字幕` as the word "subtitles"
/// would bias a semantic query toward exactly the thing the user rejected.
/// When callers did not provide parsed conditions (for example an older IPC
/// client), retain the raw-query behaviour as a backwards-compatible
/// fallback.
pub fn semantic_query_prompt_for_conditions(
    must: &[String],
    should: &[String],
    raw_query: &str,
) -> Option<String> {
    let positive_terms = must
        .iter()
        .chain(should)
        .map(String::as_str)
        .collect::<Vec<_>>();
    if positive_terms.is_empty() {
        semantic_query_prompt(raw_query)
    } else {
        semantic_query_prompt(&positive_terms.join(" "))
    }
}

fn is_han(character: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&character) || ('\u{3400}'..='\u{4dbf}').contains(&character)
}

pub fn default_runtime_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let packaged = parent.join("onnxruntime.dll");
            if packaged.is_file() {
                return packaged;
            }
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("vendor")
        .join("onnxruntime")
        .join("onnxruntime.dll")
}

fn require_ready(cache_models: &Path, runtime_path: &Path) -> AppResult<()> {
    if !cache_models.join(READY_MARKER).is_file() || !required_model_files_exist(cache_models) {
        return Err(AppError::Other(
            "本地语义模型尚未完整安装；请在设置中明确下载".to_string(),
        ));
    }
    ensure_runtime(runtime_path)
}

/// FastEmbed's Hugging Face cache contains one repository directory per paired
/// encoder. Checking their required files before creating a model prevents
/// `try_new` from silently initiating a re-download after a user deletes part
/// of the cache.
fn required_model_files_exist(cache_models: &Path) -> bool {
    repository_has_files(
        cache_models,
        "models--qdrant--clip-vit-b-32-vision",
        &["model.onnx", "preprocessor_config.json"],
    ) && repository_has_files(
        cache_models,
        "models--qdrant--clip-vit-b-32-text",
        &[
            "model.onnx",
            "tokenizer.json",
            "config.json",
            "special_tokens_map.json",
            "tokenizer_config.json",
        ],
    )
}

fn repository_has_files(cache_models: &Path, repository: &str, files: &[&str]) -> bool {
    let root = cache_models.join(repository);
    if !root.is_dir() {
        return false;
    }
    let found = walkdir::WalkDir::new(&root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| entry.file_name().to_str().map(str::to_string))
        .collect::<std::collections::HashSet<_>>();
    files.iter().all(|file| found.contains(*file))
}

fn ensure_runtime(runtime_path: &Path) -> AppResult<()> {
    if !runtime_path.is_file() {
        return Err(AppError::Other(format!(
            "未找到应用自带 ONNX Runtime: {}",
            runtime_path.display()
        )));
    }
    let result = ORT_INITIALIZED.get_or_init(|| {
        ort::init_from(runtime_path)
            .map(|builder| {
                // `commit` records the process-global ORT environment. Its
                // boolean return only reports whether this call won the race.
                builder.commit();
            })
            .map_err(|error| error.to_string())
    });
    result.clone().map_err(AppError::Other)
}

#[cfg(test)]
mod tests {
    use super::{
        semantic_query_prompt, semantic_query_prompt_for_conditions, status, PROVIDER_ID,
        READY_MARKER,
    };

    #[test]
    fn missing_runtime_and_model_is_an_explicit_degraded_state() {
        let root =
            std::env::temp_dir().join(format!("sceneweaver-semantic-{}", uuid::Uuid::new_v4()));
        let current = status(&root, &root.join("onnxruntime.dll"));
        assert!(!current.ready);
        assert!(!current.model_installed);
        assert_eq!(current.provider_id, PROVIDER_ID);
    }

    #[test]
    fn marker_is_not_enough_when_a_paired_model_file_is_missing() {
        let root =
            std::env::temp_dir().join(format!("sceneweaver-semantic-{}", uuid::Uuid::new_v4()));
        for (repo, files) in [
            (
                "models--qdrant--clip-vit-b-32-vision",
                vec!["model.onnx", "preprocessor_config.json"],
            ),
            (
                "models--qdrant--clip-vit-b-32-text",
                vec![
                    "model.onnx",
                    "tokenizer.json",
                    "config.json",
                    "special_tokens_map.json",
                    "tokenizer_config.json",
                ],
            ),
        ] {
            let snapshot = root.join(repo).join("snapshots").join("test");
            std::fs::create_dir_all(&snapshot).unwrap();
            for file in files {
                std::fs::write(snapshot.join(file), "test").unwrap();
            }
        }
        std::fs::write(root.join(READY_MARKER), "test").unwrap();
        let runtime = root.join("onnxruntime.dll");
        std::fs::write(&runtime, "test").unwrap();
        assert!(status(&root, &runtime).model_installed);
        std::fs::remove_file(
            root.join("models--qdrant--clip-vit-b-32-text")
                .join("snapshots")
                .join("test")
                .join("tokenizer.json"),
        )
        .unwrap();
        assert!(!status(&root, &runtime).model_installed);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn expands_known_chinese_creator_terms_and_rejects_unknown_han_only_input() {
        let prompt = semantic_query_prompt("角色雨夜回头的侧脸").unwrap();
        assert!(prompt.contains("character"));
        assert!(prompt.contains("rainy night"));
        assert!(prompt.contains("looking back"));
        assert!(semantic_query_prompt("不可识别的专有名词").is_none());
        assert_eq!(
            semantic_query_prompt("a red character"),
            Some("a red character".to_string())
        );
    }

    #[test]
    fn exclusion_terms_do_not_enter_a_parsed_semantic_prompt() {
        let prompt = semantic_query_prompt_for_conditions(
            &["战斗".to_string()],
            &["近景".to_string(), "粉色头发".to_string()],
            "战斗 优先 近景、粉色头发 不要 游戏 UI、字幕",
        )
        .unwrap();
        assert!(prompt.contains("battle"));
        assert!(prompt.contains("close up"));
        assert!(prompt.contains("pink hair"));
        assert!(!prompt.contains("subtitles"));
        assert!(!prompt.contains("user interface"));
    }
}
