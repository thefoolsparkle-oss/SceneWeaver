# SceneWeaver 打包说明

## 目标

- Windows 一键安装或便携版。
- 最终用户无需安装 Node.js、Python、PostgreSQL 或 Docker。
- 原始素材保留在用户本地。

## 桌面安装包

使用 Tauri v2 内置打包器：

```powershell
npm run tauri build
```

输出：

- `src-tauri/target/release/bundle/msi/*.msi`
- `src-tauri/target/release/bundle/nsis/*.exe`

当前验证状态：当前代码的 `src-tauri/target/release/sceneweaver.exe` 与最新 `src-tauri/target/release/bundle/nsis/SceneWeaver_0.1.0_x64-setup.exe` 已成功生成。`build.rs` 会在 Tauri 校验资源前，从 Cargo.lock 锁定的 `webview2-com-sys` x64 依赖暂存官方 `WebView2Loader.dll` 到构建资源路径，因此干净工作目录的 `cargo test --no-run` 和打包不再依赖先前残留的 `target` 文件；`bundle.resources` 再将它安装到 EXE 同级目录。同一资源清单也会携带官方 ONNX Runtime Windows x64 CPU 的 `onnxruntime.dll`，供可选本地语义模型使用。2026-07-19 已将当前安装器静默安装到专用临时目录，确认 EXE（58,017,555 bytes）、`WebView2Loader.dll`（160,320 bytes）和 `onnxruntime.dll`（14,897,976 bytes）三者同级，应用隐藏启动后存活超过 8 秒；测试安装目录已清理。尚未在干净 Windows 环境完成交互式安装、卸载、升级和 MSI 验证。

## 侧车依赖

### FFmpeg / ffprobe

- 优先检测用户 PATH 中的 ffprobe。
- 未检测到时提示用户下载或自动下载静态构建到应用数据目录。
- 打包时可将 FFmpeg 作为侧车资源放入 `src-tauri/binaries/`。

### 本地模型

- 默认模型体积应控制在可接受范围（< 500 MB）。
- 只有用户在设置页明确点击“下载模型”时才按需下载，提供状态和失败提示。
- SceneWeaver 当前使用配对的 CLIP 图文模型；ONNX Runtime 随安装器资源携带，模型权重不随安装器下载。
- 模型文件存放于 `{app_data_dir}/cache/models/`。

## 数据目录

- 数据库：`%LOCALAPPDATA%/SceneWeaver/sceneweaver.db`
- 缓存：`%LOCALAPPDATA%/SceneWeaver/cache/`
- 模型：`%LOCALAPPDATA%/SceneWeaver/cache/models/`
- 日志：`%LOCALAPPDATA%/SceneWeaver/logs/`

## 便携版

- 通过环境变量 `SCENEWEAVER_PORTABLE=1` 将数据目录置于可执行文件同级 `data/` 目录。
- 可在 USB 或外接硬盘上运行。

## 更新

- 使用 Tauri 更新器（可选）。
- 更新时不删除用户数据库和缓存。
