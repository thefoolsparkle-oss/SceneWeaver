# SceneWeaver 架构文档

## 总体架构

```text
Tauri Desktop Shell
        │
React + TypeScript UI (Vite)
        │ Tauri Commands / Events
Rust Core (src-tauri)
        ├─ commands:    前端可调用的 IPC 命令
        ├─ core:        业务逻辑（素材库、扫描、任务、缓存）
        ├─ models:      数据结构与 DTO
        ├─ db:          SQLite 访问与迁移
        ├─ providers:   模型 Provider 抽象（视觉/文本/OCR/ASR）
        └─ search:      索引与检索管线
Storage
        ├─ SQLite:      元数据、任务、实体、集合、按 Provider 隔离的向量
        ├─ LanceDB:     视觉/文本向量（阶段 3）
        ├─ FTS5:        文件名、OCR、转录文本（阶段 7）
        └─ cache/       缩略图、代理、关键帧、临时文件
```

## 目录结构

```
SceneWeaver/
├── package.json
├── vite.config.ts
├── tsconfig.json
├── index.html
├── src/                        # 前端
│   ├── main.tsx
│   ├── App.tsx
│   ├── api/                    # Tauri 命令封装
│   ├── components/             # 可复用组件
│   ├── pages/                  # 页面级组件
│   ├── stores/                 # Zustand 状态
│   └── types/                  # TypeScript 类型
├── src-tauri/                  # Rust 后端
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── commands/           # Tauri #[tauri::command]
│   │   ├── core/               # 业务逻辑
│   │   ├── db/                 # 数据库与迁移
│   │   ├── models/             # 结构体、DTO、错误
│   │   ├── providers/          # Provider trait 与实现
│   │   └── search/             # 检索管线
│   ├── migrations/             # SQL 迁移文件
│   └── icons/
└── docs/
```

## 核心模块职责

### core::library

- 创建、读取、更新、删除素材库。
- 管理包含/排除规则、索引模式、扫描状态。

### core::scanner

- 递归发现媒体文件。
- 增量扫描：根据指纹、大小、修改时间判断是否需要重新处理，并以数据库扫描标记识别离线文件。
- 双遍流式遍历：先统计、后处理，不将整库路径集合载入内存。
- 任务暂停、恢复、取消。

### core::fingerprint

- 快速指纹（文件名、大小、修改时间）。
- 可选完整哈希（blake3 / xxhash）。
- 中文路径、空格路径、超长路径处理。

### core::ffprobe

- 调用 ffprobe 以参数数组方式提取媒体元数据。
- 解析时长、分辨率、帧率、编码、旋转等。

### core::thumbnail

- 图片缩略图（Rust image crate / FFmpeg）。
- 视频封面与中间帧缩略图。
- 缓存到本地目录。

### core::video_derivatives

- 对已切分镜头生成代表关键帧和最多 6 秒的静音预览。
- 从关键帧计算黑帧比例、模糊程度和基础质量分。
- 仅返回位于应用缓存目录中的派生文件，FFmpeg 缺失时安全降级为无派生媒体的片段。

### core::job_queue

- 持久化任务队列。
- 状态机：pending / running / paused / completed / failed / cancelled。
- 崩溃恢复：启动时扫描未完成作业。

## 数据流

1. 用户选择文件夹 → 创建 `library`。
2. `scanner` 生成 `job` 并写入队列。
3. `job_queue` 分批消费，生成/更新 `asset`。
4. `ffprobe` 与 `thumbnail` 填充媒体元数据；列表 IPC 仅为已有缓存缩略图生成 data URL，不暴露任意本地路径。
5. 前端通过事件订阅进度。
6. 用户搜索时触发 `search` 管线，返回结果。

## 安全与隐私

- 创建素材库时仅接受存在的绝对目录并规范化路径。
- 仅可通过资产 ID 打开原文件或其所在文件夹，不能由前端提交任意路径。
- FFmpeg 使用参数数组，禁止 `shell=True`。
- 不记录 API Key。
- 云端能力默认关闭，启用前提示上传内容与成本。
