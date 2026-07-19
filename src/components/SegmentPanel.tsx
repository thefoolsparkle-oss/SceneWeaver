import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { addSegmentToDefaultSelects, copyToClipboard, listSegments, segmentPreviewDataUrl } from '@/api';
import { formatDuration, formatTimecode } from '@/lib/mediaFormat';
import type { Segment } from '@/types';

interface SegmentPanelProps {
  assetId: string;
  onClose: () => void;
}

/** A searchable video's shot-level actions, shared independently of a library page. */
export function SegmentPanel({ assetId, onClose }: SegmentPanelProps) {
  const segments = useQuery({
    queryKey: ['segments', assetId],
    queryFn: () => listSegments(assetId),
  });

  return (
    <section className="mt-8 rounded-xl border bg-white p-4 dark:bg-neutral-950">
      <div className="mb-3 flex items-center justify-between">
        <div>
          <h2 className="font-semibold">镜头片段</h2>
          <p className="mt-1 text-xs text-neutral-500">可预览、复制入点时间码，或把单个片段加入默认 Selects。</p>
        </div>
        <button type="button" onClick={onClose} className="text-sm text-neutral-500">关闭</button>
      </div>
      {segments.isLoading && <p className="text-sm text-neutral-500">加载片段中…</p>}
      {segments.isError && <p className="text-sm text-red-600">读取片段失败：{segments.error.message}</p>}
      {segments.data?.length === 0 && <p className="text-sm text-neutral-500">尚未生成镜头片段。请在视频卡片中执行“切分”。</p>}
      <div className="space-y-3">
        {segments.data?.map((segment) => <SegmentPanelCard key={segment.id} assetId={assetId} segment={segment} />)}
      </div>
    </section>
  );
}

function SegmentPanelCard({ assetId, segment }: { assetId: string; segment: Segment }) {
  const [showPreview, setShowPreview] = useState(false);
  const [selected, setSelected] = useState(false);
  const [selectionError, setSelectionError] = useState<string | null>(null);
  const preview = useQuery({
    queryKey: ['segmentPreview', assetId, segment.id],
    queryFn: () => segmentPreviewDataUrl(assetId, segment.id),
    enabled: showPreview,
  });
  const addToSelects = async () => {
    setSelectionError(null);
    try {
      await addSegmentToDefaultSelects(assetId, segment.id);
      setSelected(true);
    } catch (error) {
      setSelectionError(error instanceof Error ? error.message : '加入选片失败');
    }
  };
  const quality = segment.quality_score === null ? '待生成' : `${Math.round(segment.quality_score * 100)}%`;

  return (
    <article className="flex flex-col gap-3 rounded-lg border p-3 text-sm sm:flex-row">
      <div className="flex h-24 w-40 shrink-0 items-center justify-center overflow-hidden rounded bg-neutral-100 dark:bg-neutral-900">
        {segment.thumbnail_data_url ? <img src={segment.thumbnail_data_url} alt={`片段 ${segment.segment_index + 1} 关键帧`} className="h-full w-full object-cover" /> : <span className="text-xs text-neutral-500">未生成关键帧</span>}
      </div>
      <div className="min-w-0 flex-1">
        <p className="font-medium">#{segment.segment_index + 1} · {formatDuration(segment.duration_ms)}</p>
        <p className="mt-1 text-neutral-500">{formatTimecode(segment.start_ms)} – {formatTimecode(segment.end_ms)}</p>
        <p className="mt-1 text-xs text-neutral-500">质量 {quality} · 黑帧 {formatMetric(segment.black_frame_score)} · 模糊 {formatMetric(segment.blur_score)} · 字幕提示 {segment.subtitle_present === null ? '待分析' : segment.subtitle_present ? '有' : '无'}</p>
        <div className="mt-3 flex flex-wrap gap-2">
          <button type="button" onClick={() => copyToClipboard(formatTimecode(segment.start_ms))} className="rounded border px-2 py-1 text-xs">复制入点时间码</button>
          <button type="button" onClick={() => setShowPreview((value) => !value)} disabled={!segment.preview_path} className="rounded border px-2 py-1 text-xs disabled:opacity-50">{showPreview ? '隐藏预览' : '播放短预览'}</button>
          <button type="button" onClick={addToSelects} className="rounded border px-2 py-1 text-xs">{selected ? '已加入选片' : '加入选片'}</button>
        </div>
        {selectionError && <p className="mt-2 text-xs text-red-600">{selectionError}</p>}
        {showPreview && (preview.isLoading ? <p className="mt-2 text-xs text-neutral-500">加载预览中…</p> : preview.data ? <video controls autoPlay muted src={preview.data} className="mt-3 max-h-56 w-full rounded bg-black" /> : <p className="mt-2 text-xs text-neutral-500">短预览不可用。</p>)}
      </div>
    </article>
  );
}

function formatMetric(value: number | null | undefined) {
  return value === null || value === undefined ? '—' : `${Math.round(value * 100)}%`;
}
