# SceneWeaver

> 面向内容创作者的本地多模态素材搜索、整理与选片工作台。
> Local-first Multimodal Media Search & Selects Workspace for Creators.

## 核心价值

- **本地优先**：原始素材不搬家、不上传。
- **自然语言搜索**：输入一句话，直接定位到镜头与时间段。
- **参考图与实体**：用参考图定义角色、人物或视觉主体。
- **组合条件**：Must / Should / Must Not 让结果更可控。
- **专业流程衔接**：打开源文件、复制时间码、导出 CSV / JSON / EDL / FCPXML。

## 当前阶段

本项目正在按照 [`SceneWeaver_PRODUCT_REDESIGN.md`](./SceneWeaver_PRODUCT_REDESIGN.md) 进行 V2 重构实施。详见 [`docs/PROGRESS.md`](./docs/PROGRESS.md)。

当前可用闭环包括：本地文件夹素材库、可恢复扫描任务、图片缩略图、基础关键词 / Must / Must Not 搜索、收藏、默认选片与可编辑选片集合（备注、评分、推荐入/出点、拖拽排序）。默认和自定义集合都可导出 CSV、JSON、EDL、FCPXML 和 PNG 联系表；片段范围会被保留，已填写的推荐入/出点会优先覆盖片段范围。视频在扫描时会自动切分镜头并生成关键帧、短预览与基础黑帧/模糊质量指标（需要本机 FFmpeg）；关键帧还会以保守的下部高对比度启发式写入字幕提示，因此“字幕 / subtitle”可作为视频的 Must / Should / Must Not 条件。它不是 OCR。FFmpeg 缺失或派生失败时仍保留可检索的视频素材和完整片段，并可稍后手动重试。超时的镜头检测或派生进程会被终止并回收，避免遗留后台 FFmpeg 占用扫描流程。准确片段可直接加入选片集合。设置页提供明确触发的本地 CLIP 模型下载与重建语义索引；参考图相似检索与 Entity 的正/负参考、候选“是/非实体”本地反馈已可用。中文通用视觉语义与完整 ACG 场景理解仍在开发中，详见 [`docs/KNOWN_LIMITATIONS.md`](./docs/KNOWN_LIMITATIONS.md)。

启用 ACG Creator Pack 后，可在素材卡片上添加本地确认的标签（如“战斗”“游戏 UI”“侧脸”）。标签持久化在 SQLite 中，之后会与文件名、路径一起参与 Must / Should / Must Not 搜索；命中标签时，结果会明确标记为 ACG 标签命中而非误称为文件名命中。关键词、CLIP 与 Entity 候选会在合并后按可解释分数统一排序；同分候选会按素材库稳定轮换，避免单个库挤占前排。未启用或未标注的素材不会被强行推断标签。

FCPXML 导出会对 Windows 路径中的中文、空格和保留字符生成规范的文件 URI，并按图片、视频或音频写入对应的媒体属性。资源 ID 不会与格式 ID 冲突；时间线采用首个有效素材帧率，视频资源各自保留其已探测的帧率和分辨率（含 23.976 / 29.97 / 59.94 的 NTSC 分数帧时长）。仍建议在目标剪辑软件中以真实素材完成最终导入检查。

图片扫描会建立本地持久化视觉特征；在搜索结果点击“相似”可离线召回视觉相近的已索引图片，也可在主搜索页直接选择已创建的实体。文本搜索会识别已创建 Entity 的名称和别名，并把名称/别名命中与正/负参考图候选合并到同一结果中；Must Not 仅排除而不会触发实体召回。基础 Provider 无需下载模型；用户可在设置页明确下载 CLIP 后建立独立语义向量索引，未安装或模型文件不完整时不会自动联网。增量重扫发现源文件内容变化时，会先失效旧向量、镜头片段、片段选片和缓存派生文件，再写入当前颜色向量并同步实体素材反馈，避免旧内容被语义检索或片段导出继续使用。已安装时，关键词与 CLIP 候选会在媒体类型、质量和 Must Not 硬过滤后去重融合；语义提示只使用 Must/Should，不会把“不要字幕 / UI”等排除词反向送入 CLIP。CLIP 搜索对“角色、雨夜、侧脸、回头、战斗、字幕、近/远景”等已知中文创作词会在本机透明扩展为英文提示，未知中文仍回退关键词。配对编码器已通过真实下载与 512 维推理 smoke 验证。

启用 ACG Creator Pack 后，搜索页会显示本地“角色近景、雨夜侧脸、战斗无 UI/字幕、剧情过场、情绪结尾”预设；它们只是可编辑查询的快捷入口，未调用云端服务。

当文件名/路径没有命中时，基础 Provider 还支持“红色 / 绿色 / 蓝色 / 黄色 / 夜色”等中英文颜色词的视觉召回；未知描述会明确降级为关键词搜索，不会假装理解画面语义。

搜索页的“参考图”会打开系统文件选择器，用用户明确选中的图片在本地检索相似素材；参考图不会上传、复制或保存。

## 技术栈

- 桌面端：Tauri v2 + React + TypeScript + Vite
- 本地核心：Rust
- 业务数据库：SQLite（WAL 模式）
- 向量索引：LanceDB / SQLite 向量扩展（阶段 3 接入）
- 媒体处理：FFmpeg / ffprobe（当前开发版优先检测 PATH；发布版将打包为侧车）

## 开发环境要求

- Windows 10/11
- Node.js >= 20
- Rust（本项目使用 `x86_64-pc-windows-gnu` 工具链 + MinGW-w64）
- （可选）FFmpeg / ffprobe 已加入 PATH

## 开发启动

```powershell
npm.cmd install
npm.cmd run tauri dev
```

## 测试

```powershell
npm.cmd run lint
npm.cmd test
cargo test --manifest-path src-tauri/Cargo.toml
```

当前 GNU/Tauri 开发环境会使 Rust 测试二进制在启动时失败；请以 `cargo test --no-run --manifest-path src-tauri/Cargo.toml` 验证 Rust 测试编译，完整限制见 [`docs/KNOWN_LIMITATIONS.md`](./docs/KNOWN_LIMITATIONS.md)。

推送到 `main` 或创建 Pull Request 时，[`.github/workflows/ci.yml`](./.github/workflows/ci.yml) 会在 Windows Runner 自动执行前端 lint/test/build、Rust 格式与测试编译，以及不依赖模型下载的 `core_smoke`。2026-07-19 已在干净 Windows Runner 实际通过完整质量门禁；后续结果以 GitHub Actions 页面为准。

如需运行不依赖窗口的真实核心集成验证（会在系统临时目录创建并清理测试图片），执行：

```powershell
cargo run --manifest-path src-tauri/Cargo.toml --bin core_smoke
```

## 打包

```powershell
npm.cmd run tauri build
```

## 许可证

MIT（以仓库根目录 LICENSE 文件为准）
