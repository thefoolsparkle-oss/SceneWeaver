import { useEffect, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { save } from '@tauri-apps/plugin-dialog';
import {
  createSelectCollection, exportDefaultSelectsCsv, exportDefaultSelectsEdl,
  exportDefaultSelectsFcpxml, exportDefaultSelectsJson, exportSelectCollectionCsv,
  exportSelectCollectionContactSheet, exportSelectCollectionContactSheetHtml, exportSelectCollectionEdl, exportSelectCollectionFcpxml, exportSelectCollectionJson,
  listSelectCollections, listSelectItems, moveSelectItem, removeSelectItem,
  reorderSelectItem, updateSelectItem,
} from '@/api';
import { formatDuration, formatTimecode } from '@/lib/mediaFormat';
import type { SelectItem } from '@/types';

const defaultCollectionName = '我的选片';
type ExportFormat = 'csv' | 'json' | 'edl' | 'fcpxml' | 'png' | 'html';

export default function Selects() {
  const queryClient = useQueryClient();
  const [collectionId, setCollectionId] = useState<string | null>(null);
  const [newName, setNewName] = useState('');
  const [newDescription, setNewDescription] = useState('');
  const [exportError, setExportError] = useState<string | null>(null);
  const [draggedItemId, setDraggedItemId] = useState<string | null>(null);
  const collections = useQuery({ queryKey: ['selectCollections'], queryFn: listSelectCollections });
  const items = useQuery({ queryKey: ['selectItems', collectionId], queryFn: () => listSelectItems(collectionId!), enabled: collectionId !== null });
  const activeCollection = collections.data?.find((collection) => collection.id === collectionId);

  useEffect(() => {
    if (collections.data?.length && !collections.data.some((collection) => collection.id === collectionId)) setCollectionId(collections.data[0].id);
  }, [collections.data, collectionId]);

  const invalidateItems = () => queryClient.invalidateQueries({ queryKey: ['selectItems'] });
  const create = useMutation({
    mutationFn: createSelectCollection,
    onSuccess: (collection) => {
      setNewName(''); setNewDescription(''); setCollectionId(collection.id);
      queryClient.invalidateQueries({ queryKey: ['selectCollections'] });
    },
  });
  const remove = useMutation({ mutationFn: removeSelectItem, onSuccess: invalidateItems });
  const move = useMutation({
    mutationFn: ({ itemId, targetId }: { itemId: string; targetId: string }) => moveSelectItem(itemId, targetId),
    onSuccess: invalidateItems,
  });
  const reorder = useMutation({
    mutationFn: ({ itemId, position }: { itemId: string; position: number }) => reorderSelectItem(itemId, position),
    onSuccess: invalidateItems,
  });

  const exportFile = async (format: ExportFormat) => {
    if (!activeCollection) return;
    const path = await save({ defaultPath: `sceneweaver-selects.${format}`, filters: [{ name: format.toUpperCase(), extensions: [format] }] });
    if (!path) return;
    try {
      if (format === 'png') {
        await exportSelectCollectionContactSheet(activeCollection.id, path);
      } else if (format === 'html') {
        await exportSelectCollectionContactSheetHtml(activeCollection.id, path);
      } else if (activeCollection.name === defaultCollectionName) {
        const operation = format === 'csv' ? exportDefaultSelectsCsv : format === 'json' ? exportDefaultSelectsJson : format === 'edl' ? exportDefaultSelectsEdl : exportDefaultSelectsFcpxml;
        await operation(path);
      } else {
        const operation = format === 'csv' ? exportSelectCollectionCsv : format === 'json' ? exportSelectCollectionJson : format === 'edl' ? exportSelectCollectionEdl : exportSelectCollectionFcpxml;
        await operation(activeCollection.id, path);
      }
      setExportError(null);
    } catch (error) {
      setExportError(error instanceof Error ? error.message : '导出失败');
    }
  };

  const handleDrop = (targetItemId: string) => {
    if (!draggedItemId || draggedItemId === targetItemId || !items.data) return;
    const targetPosition = items.data.findIndex((item) => item.id === targetItemId);
    if (targetPosition >= 0) reorder.mutate({ itemId: draggedItemId, position: targetPosition });
    setDraggedItemId(null);
  };

  return <div className="p-8">
    <div className="mb-6 flex flex-wrap items-center justify-between gap-3">
      <div>
        <h1 className="text-2xl font-bold">我的选片</h1>
        <p className="mt-1 text-sm text-neutral-500">从素材库或搜索结果加入默认集合；在这里分组、排序、评分、标注并导出。</p>
      </div>
      {activeCollection && <div className="flex flex-wrap gap-2">
        {(['csv', 'json', 'edl', 'fcpxml', 'png', 'html'] as ExportFormat[]).map((format) => <ExportButton key={format} label={format === 'png' ? '联系表 PNG' : format === 'html' ? '联系表 HTML' : format.toUpperCase()} onClick={() => void exportFile(format)} disabled={!items.data?.length} />)}
      </div>}
    </div>
    {exportError && <p className="mb-3 text-sm text-red-600">导出失败：{exportError}</p>}
    {reorder.isError && <p className="mb-3 text-sm text-red-600">排序失败：{reorder.error.message}</p>}
    <div className="grid gap-6 lg:grid-cols-[17rem_minmax(0,1fr)]">
      <aside className="rounded-xl border p-4">
        <h2 className="font-medium">选片集合</h2>
        <div className="mt-3 space-y-1">{collections.data?.map((collection) => <button key={collection.id} onClick={() => setCollectionId(collection.id)} className={`w-full rounded-lg px-3 py-2 text-left text-sm ${collection.id === collectionId ? 'bg-brand-50 text-brand-700 dark:bg-brand-950' : 'hover:bg-neutral-100 dark:hover:bg-neutral-900'}`}>{collection.name}</button>)}</div>
        <form className="mt-5 space-y-2 border-t pt-4" onSubmit={(event) => { event.preventDefault(); if (newName.trim()) create.mutate({ name: newName, description: newDescription }); }}>
          <input value={newName} onChange={(event) => setNewName(event.target.value)} placeholder="新集合名称" maxLength={80} className="w-full rounded border px-3 py-2 text-sm" />
          <input value={newDescription} onChange={(event) => setNewDescription(event.target.value)} placeholder="说明（可选）" className="w-full rounded border px-3 py-2 text-sm" />
          <button disabled={create.isPending} className="w-full rounded bg-brand-600 px-3 py-2 text-sm text-white disabled:opacity-50">新建集合</button>
          {create.isError && <p className="text-xs text-red-600">{create.error.message}</p>}
        </form>
      </aside>
      <section>{activeCollection ? <>
        <div className="mb-3"><h2 className="font-medium">{activeCollection.name}</h2>{activeCollection.description && <p className="text-sm text-neutral-500">{activeCollection.description}</p>}<p className="mt-1 text-xs text-neutral-500">拖拽卡片即可排序；上下按钮可作为键盘替代。导出会保留当前顺序，推荐入/出点优先于片段范围。</p></div>
        {items.isLoading && <p className="text-neutral-500">加载中…</p>}
        {items.isError && <p className="text-red-600">加载选片失败。</p>}
        {items.data?.length === 0 && <p className="rounded-xl border border-dashed p-8 text-center text-neutral-500">此集合暂无选片。</p>}
        <div className="space-y-3">{items.data?.map((item, index) => <SelectItemCard key={item.id} item={item} index={index} total={items.data?.length ?? 0} collections={collections.data ?? []} currentCollectionId={collectionId} isDragging={draggedItemId === item.id} onDragStart={() => setDraggedItemId(item.id)} onDragEnd={() => setDraggedItemId(null)} onDrop={() => handleDrop(item.id)} onSave={(request) => updateSelectItem(item.id, request).then(invalidateItems)} onRemove={() => remove.mutate(item.id)} onMove={(targetId) => move.mutate({ itemId: item.id, targetId })} onReorder={(position) => reorder.mutate({ itemId: item.id, position })} />)}</div>
      </> : <p className="rounded-xl border border-dashed p-8 text-center text-neutral-500">创建集合，或从媒体卡片加入第一条选片。</p>}</section>
    </div>
  </div>;
}

function SelectItemCard({ item, index, total, collections, currentCollectionId, isDragging, onDragStart, onDragEnd, onDrop, onSave, onRemove, onMove, onReorder }: { item: SelectItem; index: number; total: number; collections: { id: string; name: string }[]; currentCollectionId: string | null; isDragging: boolean; onDragStart: () => void; onDragEnd: () => void; onDrop: () => void; onSave: (request: { rating: number | null; note: string | null; recommended_in_ms: number | null; recommended_out_ms: number | null }) => Promise<void>; onRemove: () => void; onMove: (collectionId: string) => void; onReorder: (position: number) => void }) {
  const [rating, setRating] = useState(item.rating?.toString() ?? '');
  const [note, setNote] = useState(item.note ?? '');
  const [inPoint, setInPoint] = useState(item.recommended_in_ms === null ? '' : (item.recommended_in_ms / 1000).toString());
  const [outPoint, setOutPoint] = useState(item.recommended_out_ms === null ? '' : (item.recommended_out_ms / 1000).toString());
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const saveItem = async () => {
    const milliseconds = (value: string) => value.trim() === '' ? null : Math.round(Number(value) * 1000);
    const parsedRating = rating.trim() === '' ? null : Number(rating);
    const start = milliseconds(inPoint); const end = milliseconds(outPoint);
    if (!Number.isInteger(parsedRating ?? 0) || (start !== null && (!Number.isFinite(start) || start < 0)) || (end !== null && (!Number.isFinite(end) || end < 0))) { setError('评分需为整数，入/出点需为非负秒数。'); return; }
    setSaving(true); setError(null);
    try { await onSave({ rating: parsedRating, note: note.trim() || null, recommended_in_ms: start, recommended_out_ms: end }); } catch (cause) { setError(cause instanceof Error ? cause.message : '保存失败'); } finally { setSaving(false); }
  };
  return <article draggable onDragStart={(event) => { event.dataTransfer.effectAllowed = 'move'; onDragStart(); }} onDragEnd={onDragEnd} onDragOver={(event) => event.preventDefault()} onDrop={(event) => { event.preventDefault(); onDrop(); }} className={`flex flex-col gap-3 rounded-xl border bg-white p-3 dark:bg-neutral-950 sm:flex-row ${isDragging ? 'opacity-50 ring-2 ring-brand-400' : ''}`}>
    <div className="flex h-24 w-40 shrink-0 items-center justify-center overflow-hidden rounded bg-neutral-100 dark:bg-neutral-900">{item.segment?.thumbnail_data_url ? <img src={item.segment.thumbnail_data_url} alt="" className="h-full w-full object-cover" /> : item.asset.thumbnail_data_url ? <img src={item.asset.thumbnail_data_url} alt="" className="h-full w-full object-cover" /> : <span>{item.asset.media_type === 'video' ? '🎬' : '🖼️'}</span>}</div>
    <div className="min-w-0 flex-1"><p className="truncate font-medium" title={item.asset.file_path}>{item.asset.file_name}</p><p className="mb-2 text-xs text-neutral-500">{item.segment ? `镜头 #${item.segment.segment_index + 1} · ${formatTimecode(item.segment.start_ms)} — ${formatTimecode(item.segment.end_ms)} · ${formatDuration(item.segment.duration_ms)}` : `${formatDuration(item.asset.duration_ms)} · ${item.asset.file_path}`}</p>
      <div className="grid gap-2 sm:grid-cols-4"><label className="text-xs">评分<input value={rating} onChange={(event) => setRating(event.target.value)} inputMode="numeric" className="mt-1 w-full rounded border px-2 py-1 text-sm" placeholder="0–5" /></label><label className="text-xs">推荐入点（秒）<input value={inPoint} onChange={(event) => setInPoint(event.target.value)} inputMode="decimal" className="mt-1 w-full rounded border px-2 py-1 text-sm" /></label><label className="text-xs">推荐出点（秒）<input value={outPoint} onChange={(event) => setOutPoint(event.target.value)} inputMode="decimal" className="mt-1 w-full rounded border px-2 py-1 text-sm" /></label><label className="text-xs">移动到<select value={currentCollectionId ?? ''} onChange={(event) => { if (event.target.value !== currentCollectionId) onMove(event.target.value); }} className="mt-1 w-full rounded border px-2 py-1 text-sm">{collections.map((collection) => <option value={collection.id} key={collection.id}>{collection.name}</option>)}</select></label></div>
      <label className="mt-2 block text-xs">备注<textarea value={note} onChange={(event) => setNote(event.target.value)} className="mt-1 min-h-14 w-full rounded border px-2 py-1 text-sm" /></label>{error && <p className="mt-1 text-xs text-red-600">{error}</p>}
      <div className="mt-2 flex gap-2"><button onClick={saveItem} disabled={saving} className="rounded bg-brand-600 px-3 py-1 text-sm text-white disabled:opacity-50">{saving ? '保存中…' : '保存标注'}</button><button onClick={() => onReorder(index - 1)} disabled={index === 0 || isDragging} className="rounded border px-3 py-1 text-sm disabled:opacity-50">上移</button><button onClick={() => onReorder(index + 1)} disabled={index === total - 1 || isDragging} className="rounded border px-3 py-1 text-sm disabled:opacity-50">下移</button><button onClick={onRemove} className="rounded border px-3 py-1 text-sm">移除</button></div>
    </div>
  </article>;
}

function ExportButton({ label, onClick, disabled }: { label: string; onClick: () => void; disabled: boolean }) { return <button onClick={onClick} disabled={disabled} className="rounded-lg border px-3 py-2 text-sm disabled:opacity-50">导出 {label}</button>; }
