# SceneWeaver 搜索管线

## 目标

支持自然语言、参考图、实体与组合条件的多路召回与可解释排序。

## 查询结构

```typescript
interface QuerySpec {
  raw_query: string;
  must: Condition[];
  should: Condition[];
  must_not: Condition[];
  text_terms: string[];
  reference_asset_ids: string[];
  entity_ids: string[];
  library_ids: string[];
  date_range?: DateRange;
  preferred_duration_ms?: Range;
  min_quality_score?: number;
  media_types: ('image' | 'video' | 'audio')[];
}
```

## 四层检索管线

### 第 1 层：确定性过滤

- 素材库范围
- 媒体类型
- 日期范围
- 路径匹配
- 用户标签
- 字幕/UI/模糊/黑帧等硬排除
- 时长范围

### 第 2 层：多路召回

并行执行：

- 视觉语义向量（CLIP 类本地模型）
- 无下载模型时使用本地颜色直方图特征，为“相似”图片召回提供确定性降级；它不替代自然语言视觉语义模型。
- 对红、绿、蓝、黄和夜色等透明、有限的颜色词，fallback 会将文本映射至同一特征空间；其他文本继续由文件名/路径关键词处理。
- 参考图向量相似度
- 实体向量相似度
- 文件名/路径全文检索（FTS5）
- OCR 文本（阶段 7）
- ASR 转录（阶段 7）

### 第 3 层：融合排序

综合计算：

- 语义相关度
- Must / Should 命中数
- Must Not 违反数
- 质量分
- 重复惩罚
- 多样性
- 镜头长度适配

### 第 4 层：可选精确重排

- 仅对 Top 20~50 候选执行
- 用户主动开启云端模型时显示成本与上传数量
- 本地更强模型作为默认

## 结果解释

每个结果返回：

- 匹配分数
- 匹配原因列表
- 未满足条件列表
- 质量提示
- 排除原因（如被 must_not 过滤）

当前实现已将关键词 Must、Should 和颜色视觉 fallback 的实际命中原因、以及未命中的 Should 条件返回给界面。Must Not 属于硬过滤，因此不会出现在结果中；其过滤说明将在下一切片的查询详情中展示。

当前界面已提供图片、视频和音频的结构化范围筛选，并由 SQLite 在召回前处理。

最低质量阈值会应用于视频的持久化镜头质量分；图片不因尚无镜头质量数据而被排除。

## 去重策略

- 相邻镜头去重
- 同源视频时间窗口去重
- 视觉近重复惩罚
- 同一场景最大结果数限制
