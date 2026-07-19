# SceneWeaver 数据库设计

## 数据库

- 引擎：SQLite 3
- 模式：WAL（Write-Ahead Logging）
- 文件：`{app_data_dir}/sceneweaver.db`
- 迁移：按版本号顺序执行的 `.sql` 文件

## 表结构

### libraries

| 字段 | 类型 | 说明 |
|---|---|---|
| id | TEXT PRIMARY KEY | UUID v4 |
| name | TEXT NOT NULL | 素材库名称 |
| root_path | TEXT NOT NULL | 根目录规范化绝对路径 |
| status | TEXT | idle / scanning / paused / error |
| index_profile | TEXT | quick / balanced / precise |
| include_patterns | TEXT (JSON) | 包含规则 |
| exclude_patterns | TEXT (JSON) | 排除规则 |
| watch_enabled | INTEGER | 0/1 |
| last_scan_at | INTEGER | Unix ms |
| created_at | INTEGER | Unix ms |
| updated_at | INTEGER | Unix ms |

### assets

| 字段 | 类型 | 说明 |
|---|---|---|
| id | TEXT PRIMARY KEY | UUID v4 |
| library_id | TEXT NOT NULL | 外键 |
| media_type | TEXT | image / video / audio |
| file_path | TEXT NOT NULL | 原始绝对路径 |
| normalized_path | TEXT NOT NULL | 规范化路径；与 library_id 组成唯一键 |
| file_name | TEXT | 文件名 |
| extension | TEXT | 小写扩展名 |
| size_bytes | INTEGER | 文件大小 |
| modified_at | INTEGER | 修改时间 ms |
| quick_fingerprint | TEXT | 快速指纹 |
| full_hash | TEXT | 可选完整哈希 |
| duration_ms | INTEGER | 时长 |
| width | INTEGER | 宽度 |
| height | INTEGER | 高度 |
| fps | REAL | 帧率 |
| codec | TEXT | 编码 |
| capture_time | INTEGER | 拍摄时间 |
| status | TEXT | pending / indexed / error / offline |
| index_level | INTEGER | 0-4 |
| analysis_version | INTEGER | 分析版本 |
| last_seen_scan_at | INTEGER | 最近一次成功发现该文件的扫描标记 |
| created_at | INTEGER | Unix ms |
| updated_at | INTEGER | Unix ms |

### segments

| 字段 | 类型 | 说明 |
|---|---|---|
| id | TEXT PRIMARY KEY | UUID v4 |
| asset_id | TEXT | 外键 |
| segment_type | TEXT | shot / keyframe / scene |
| segment_index | INTEGER | 序号 |
| start_ms | INTEGER | 开始时间 |
| end_ms | INTEGER | 结束时间 |
| duration_ms | INTEGER | 时长 |
| representative_frame_path | TEXT | 代表帧缓存路径 |
| thumbnail_path | TEXT | 缩略图路径 |
| preview_path | TEXT | 预览路径 |
| quality_score | REAL | 质量分 |
| subtitle_present | INTEGER | 0/1 |
| game_ui | INTEGER | nullable 0/1；保守双下角 HUD 提示 |
| black_frame_score | REAL | 黑帧分数 |
| blur_score | REAL | 模糊分数 |
| embedding_ref | TEXT | 向量引用 |
| created_at | INTEGER | Unix ms |
| updated_at | INTEGER | Unix ms |

### asset_embeddings

| 字段 | 类型 | 说明 |
|---|---|---|
| asset_id | TEXT PRIMARY KEY | 已索引图片或具有代表帧的视频资产 ID |
| provider_id | TEXT | 特征 Provider，例如 `local-color-histogram` |
| model_version | TEXT | Provider / 模型版本 |
| vector_json | TEXT | 本地持久化向量 JSON |
| created_at / updated_at | INTEGER | Unix ms |

### tags

| 字段 | 类型 | 说明 |
|---|---|---|
| id | TEXT PRIMARY KEY | UUID v4 |
| scope_type | TEXT | library / asset / segment |
| scope_id | TEXT | 外键 |
| namespace | TEXT | 命名空间 |
| key | TEXT | 标签键 |
| value | TEXT | 标签值 |
| confidence | REAL | 置信度 |
| source | TEXT | 来源 |
| pack_id | TEXT | 场景包 |
| user_confirmed | INTEGER | 0/1 |
| created_at | INTEGER | Unix ms |
| updated_at | INTEGER | Unix ms |

### entities

| 字段 | 类型 | 说明 |
|---|---|---|
| id | TEXT PRIMARY KEY | UUID v4 |
| entity_type | TEXT | person / character / product / place / custom |
| name | TEXT | 名称 |
| description | TEXT | 描述 |
| aliases_json | TEXT | 别名 JSON |
| pack_id | TEXT | 场景包 |
| created_at | INTEGER | Unix ms |
| updated_at | INTEGER | Unix ms |

### entity_references

| 字段 | 类型 | 说明 |
|---|---|---|
| id | TEXT PRIMARY KEY | UUID v4 |
| entity_id | TEXT | 外键 |
| asset_id | TEXT | nullable |
| image_path | TEXT | nullable |
| embedding_ref | TEXT | 向量引用 |
| is_positive | INTEGER | 0/1 |
| created_at | INTEGER | Unix ms |

`embedding_ref` 当前保存本地视觉特征 JSON；参考图原文件不会复制到应用数据目录。

### searches

| 字段 | 类型 | 说明 |
|---|---|---|
| id | TEXT PRIMARY KEY | UUID v4 |
| raw_query | TEXT | 原始查询 |
| parsed_query_json | TEXT | 解析后 JSON |
| result_count | INTEGER | 结果数 |
| latency_ms | INTEGER | 延迟 |
| created_at | INTEGER | Unix ms |

### selects_collections

| 字段 | 类型 | 说明 |
|---|---|---|
| id | TEXT PRIMARY KEY | UUID v4 |
| name | TEXT | 名称 |
| description | TEXT | 描述 |
| created_at | INTEGER | Unix ms |
| updated_at | INTEGER | Unix ms |

### selects_items

| 字段 | 类型 | 说明 |
|---|---|---|
| id | TEXT PRIMARY KEY | UUID v4 |
| collection_id | TEXT | 外键 |
| asset_id | TEXT | 外键 |
| segment_id | TEXT | nullable |
| position | INTEGER | 排序位置 |
| rating | INTEGER | 评分 0-5 |
| note | TEXT | 备注 |
| recommended_in_ms | INTEGER | 推荐入点 |
| recommended_out_ms | INTEGER | 推荐出点 |
| created_at | INTEGER | Unix ms |
| updated_at | INTEGER | Unix ms |

`selects_items` 已在选片页用于自定义集合归档、评分、备注与推荐入/出点；当前 `position` 按插入/移动顺序维护，拖拽重排尚未接入。

### jobs

| 字段 | 类型 | 说明 |
|---|---|---|
| id | TEXT PRIMARY KEY | UUID v4 |
| job_type | TEXT | scan / thumbnail / shot_detect / index |
| library_id | TEXT | nullable |
| asset_id | TEXT | nullable |
| status | TEXT | pending / running / paused / completed / failed / cancelled |
| priority | INTEGER | 优先级 |
| progress | REAL | 0-1 |
| current_step | TEXT | 当前步骤 |
| checkpoint_json | TEXT | 检查点 JSON |
| error_code | TEXT | 错误码 |
| error_message | TEXT | 错误信息 |
| started_at | INTEGER | Unix ms |
| finished_at | INTEGER | Unix ms |
| created_at | INTEGER | Unix ms |
| updated_at | INTEGER | Unix ms |

### favorites

| 字段 | 类型 | 说明 |
|---|---|---|
| asset_id | TEXT PRIMARY KEY | 被收藏的素材 ID |
| created_at | INTEGER | 收藏时间 |

## 索引

- `assets(library_id, status)`
- `assets(library_id, normalized_path)`（唯一；同一原文件可属于多个素材库）
- `assets(quick_fingerprint)`
- `segments(asset_id)`
- `asset_embeddings(provider_id, model_version)`
- `tags(scope_type, scope_id)`
- `entity_references(entity_id)`
- `jobs(status, job_type)`
- `jobs(library_id, status)`
