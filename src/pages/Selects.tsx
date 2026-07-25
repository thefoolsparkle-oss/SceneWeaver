import { useEffect, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { save } from '@tauri-apps/plugin-dialog';
import {
  createSelectCollection, exportDefaultSelectsCsv, exportDefaultSelectsEdl,
  exportDefaultSelectsFcpxml, exportDefaultSelectsJson, exportSelectCollectionCsv,
  exportSelectCollectionContactSheet, exportSelectCollectionContactSheetHtml, exportSelectCollectionEdl, exportSelectCollectionFcpxml, exportSelectCollectionJson,
  addSelectItemTag, addSelectItemTagBatch, listSelectCollections, listSelectItems, moveSelectItem, openAsset, removeSelectItem, removeSelectItemTag,
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
  const [tagFilter, setTagFilter] = useState('');
  const [selectedItemIds, setSelectedItemIds] = useState<Set<string>>(new Set());
  const [batchTag, setBatchTag] = useState('');
  const collections = useQuery({ queryKey: ['selectCollections'], queryFn: listSelectCollections });
  const items = useQuery({ queryKey: ['selectItems', collectionId], queryFn: () => listSelectItems(collectionId!), enabled: collectionId !== null });
  const activeCollection = collections.data?.find((collection) => collection.id === collectionId);

  useEffect(() => {
    if (collections.data?.length && !collections.data.some((collection) => collection.id === collectionId)) setCollectionId(collections.data[0].id);
  }, [collections.data, collectionId]);
  useEffect(() => setSelectedItemIds(new Set()), [collectionId]);

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
  const batchAddTag = useMutation({
    mutationFn: ({ itemIds, value }: { itemIds: string[]; value: string }) => addSelectItemTagBatch(itemIds, value),
    onSuccess: () => { setBatchTag(''); setSelectedItemIds(new Set()); return invalidateItems(); },
  });
  const batchOpen = useMutation({
    mutationFn: async (itemIds: string[]) => {
      const assetIds = [...new Set(items.data?.filter((item) => itemIds.includes(item.id)).map((item) => item.asset_id) ?? [])];
      await Promise.all(assetIds.map((assetId) => openAsset(assetId)));
      return assetIds.length;
    },
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
  const visibleItems = items.data?.filter((item) => !tagFilter.trim() || item.tags.some((tag) => tag.toLocaleLowerCase().includes(tagFilter.trim().toLocaleLowerCase()))) ?? [];
  const allVisibleSelected = visibleItems.length > 0 && visibleItems.every((item) => selectedItemIds.has(item.id));

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
        {!!items.data?.length && <div className="mb-3 flex flex-wrap items-center gap-2 rounded-lg border p-3"><input value={tagFilter} onChange={(event) => setTagFilter(event.target.value)} placeholder="按标签筛选" aria-label="按标签筛选" className="min-w-40 flex-1 rounded border px-2 py-1 text-sm" /><button type="button" onClick={() => setSelectedItemIds(allVisibleSelected ? new Set() : new Set(visibleItems.map((item) => item.id)))} className="rounded border px-2 py-1 text-xs">{allVisibleSelected ? '取消全选' : `全选筛选结果 (${visibleItems.length})`}</button></div>}
        {selectedItemIds.size > 0 && <form onSubmit={(event) => { event.preventDefault(); if (batchTag.trim()) batchAddTag.mutate({ itemIds: [...selectedItemIds], value: batchTag }); }} className="mb-3 flex flex-wrap items-center gap-2 rounded-lg border border-brand-200 bg-brand-50 p-3 text-sm text-brand-800"><span>已选择 {selectedItemIds.size} 项</span><input value={batchTag} onChange={(event) => setBatchTag(event.target.value)} maxLength={80} placeholder="批量添加标签" aria-label="批量添加标签" className="rounded border border-brand-200 bg-white px-2 py-1 text-sm text-neutral-800" /><button disabled={batchAddTag.isPending || !batchTag.trim()} className="rounded bg-brand-600 px-3 py-1 text-white disabled:opacity-50">{batchAddTag.isPending ? '添加中…' : '添加标签'}</button><button type="button" onClick={() => batchOpen.mutate([...selectedItemIds])} disabled={batchOpen.isPending} className="rounded border px-3 py-1 disabled:opacity-50">{batchOpen.isPending ? '打开中…' : '批量打开源素材'}</button><button type="button" onClick={() => setSelectedItemIds(new Set())} className="rounded border px-3 py-1">取消选择</button>{(batchAddTag.isError || batchOpen.isError) && <span className="text-red-600">{(batchAddTag.error ?? batchOpen.error)?.message}</span>}</form>}
        {items.data?.length !== 0 && visibleItems.length === 0 && <p className="rounded-xl border border-dashed p-6 text-center text-neutral-500">没有匹配该标签的选片。</p>}
        <div className="space-y-3">{visibleItems.map((item, index) => <SelectItemCard key={item.id} item={item} index={items.data?.findIndex((candidate) => candidate.id === item.id) ?? index} total={items.data?.length ?? 0} collections={collections.data ?? []} currentCollectionId={collectionId} selected={selectedItemIds.has(item.id)} isDragging={draggedItemId === item.id} onToggleSelection={() => setSelectedItemIds((current) => { const next = new Set(current); if (next.has(item.id)) next.delete(item.id); else next.add(item.id); return next; })} onDragStart={() => setDraggedItemId(item.id)} onDragEnd={() => setDraggedItemId(null)} onDrop={() => handleDrop(item.id)} onSave={(request) => updateSelectItem(item.id, request).then(invalidateItems)} onAddTag={(value) => addSelectItemTag(item.id, value).then(invalidateItems)} onRemoveTag={(value) => removeSelectItemTag(item.id, value).then(invalidateItems)} onRemove={() => remove.mutate(item.id)} onMove={(targetId) => move.mutate({ itemId: item.id, targetId })} onReorder={(position) => reorder.mutate({ itemId: item.id, position })} />)}</div>
      </> : <p className="rounded-xl border border-dashed p-8 text-center text-neutral-500">创建集合，或从媒体卡片加入第一条选片。</p>}</section>
    </div>
  </div>;
}

function SelectItemCard({ item, index, total, collections, currentCollectionId, selected, isDragging, onToggleSelection, onDragStart, onDragEnd, onDrop, onSave, onAddTag, onRemoveTag, onRemove, onMove, onReorder }: { item: SelectItem; index: number; total: number; collections: { id: string; name: string }[]; currentCollectionId: string | null; selected: boolean; isDragging: boolean; onToggleSelection: () => void; onDragStart: () => void; onDragEnd: () => void; onDrop: () => void; onSave: (request: { rating: number | null; note: string | null; recommended_in_ms: number | null; recommended_out_ms: number | null }) => Promise<void>; onAddTag: (value: string) => Promise<void>; onRemoveTag: (value: string) => Promise<void>; onRemove: () => void; onMove: (collectionId: string) => void; onReorder: (position: number) => void }) {
  const [rating, setRating] = useState(item.rating?.toString() ?? '');
  const [note, setNote] = useState(item.note ?? '');
  const [inPoint, setInPoint] = useState(item.recommended_in_ms === null ? '' : (item.recommended_in_ms / 1000).toString());
  const [outPoint, setOutPoint] = useState(item.recommended_out_ms === null ? '' : (item.recommended_out_ms / 1000).toString());
  const [tagInput, setTagInput] = useState('');
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
  const addTag = async (event: React.FormEvent) => {
    event.preventDefault();
    const value = tagInput.trim();
    if (!value) return;
    setSaving(true); setError(null);
    try { await onAddTag(value); setTagInput(''); } catch (cause) { setError(cause instanceof Error ? cause.message : '添加标签失败'); } finally { setSaving(false); }
  };
  const removeTag = async (value: string) => {
    setSaving(true); setError(null);
    try { await onRemoveTag(value); } catch (cause) { setError(cause instanceof Error ? cause.message : '移除标签失败'); } finally { setSaving(false); }
  };
  return <article draggable onDragStart={(event) => { event.dataTransfer.effectAllowed = 'move'; onDragStart(); }} onDragEnd={onDragEnd} onDragOver={(event) => event.preventDefault()} onDrop={(event) => { event.preventDefault(); onDrop(); }} className={`flex flex-col gap-3 rounded-xl border bg-white p-3 dark:bg-neutral-950 sm:flex-row ${isDragging ? 'opacity-50 ring-2 ring-brand-400' : ''}`}>
    <div className="flex h-24 w-40 shrink-0 items-center justify-center overflow-hidden rounded bg-neutral-100 dark:bg-neutral-900">{item.segment?.thumbnail_data_url ? <img src={item.segment.thumbnail_data_url} alt="" className="h-full w-full object-cover" /> : item.asset.thumbnail_data_url ? <img src={item.asset.thumbnail_data_url} alt="" className="h-full w-full object-cover" /> : <span>{item.asset.media_type === 'video' ? '🎬' : '🖼️'}</span>}</div>
    <div className="min-w-0 flex-1"><div className="flex items-center gap-2"><input type="checkbox" checked={selected} onChange={onToggleSelection} aria-label={`选择 ${item.asset.file_name}`} /><p className="truncate font-medium" title={item.asset.file_path}>{item.asset.file_name}</p></div><p className="mb-2 text-xs text-neutral-500">{item.segment ? `镜头 #${item.segment.segment_index + 1} · ${formatTimecode(item.segment.start_ms)} — ${formatTimecode(item.segment.end_ms)} · ${formatDuration(item.segment.duration_ms)}` : `${formatDuration(item.asset.duration_ms)} · ${item.asset.file_path}`}</p>
      <div className="grid gap-2 sm:grid-cols-4"><label className="text-xs">评分<input value={rating} onChange={(event) => setRating(event.target.value)} inputMode="numeric" className="mt-1 w-full rounded border px-2 py-1 text-sm" placeholder="0–5" /></label><label className="text-xs">推荐入点（秒）<input value={inPoint} onChange={(event) => setInPoint(event.target.value)} inputMode="decimal" className="mt-1 w-full rounded border px-2 py-1 text-sm" /></label><label className="text-xs">推荐出点（秒）<input value={outPoint} onChange={(event) => setOutPoint(event.target.value)} inputMode="decimal" className="mt-1 w-full rounded border px-2 py-1 text-sm" /></label><label className="text-xs">移动到<select value={currentCollectionId ?? ''} onChange={(event) => { if (event.target.value !== currentCollectionId) onMove(event.target.value); }} className="mt-1 w-full rounded border px-2 py-1 text-sm">{collections.map((collection) => <option value={collection.id} key={collection.id}>{collection.name}</option>)}</select></label></div>
      <label className="mt-2 block text-xs">备注<textarea value={note} onChange={(event) => setNote(event.target.value)} className="mt-1 min-h-14 w-full rounded border px-2 py-1 text-sm" /></label>{error && <p className="mt-1 text-xs text-red-600">{error}</p>}
      <div className="mt-2"><p className="text-xs">标签</p><div className="mt-1 flex flex-wrap gap-1">{item.tags.map((tag) => <span key={tag} className="inline-flex items-center rounded-full bg-brand-50 px-2 py-0.5 text-xs text-brand-800 dark:bg-brand-950 dark:text-brand-200">{tag}<button type="button" onClick={() => void removeTag(tag)} disabled={saving} aria-label={`移除标签 ${tag}`} className="ml-1 disabled:opacity-50">×</button></span>)}</div><form onSubmit={addTag} className="mt-1 flex gap-2"><input value={tagInput} onChange={(event) => setTagInput(event.target.value)} maxLength={80} placeholder="添加标签" aria-label="添加标签" className="min-w-0 flex-1 rounded border px-2 py-1 text-sm" /><button disabled={saving || !tagInput.trim()} className="rounded border px-2 py-1 text-xs disabled:opacity-50">添加</button></form></div>{error && <p className="mt-1 text-xs text-red-600">{error}</p>}
      <div className="mt-2 flex gap-2"><button onClick={saveItem} disabled={saving} className="rounded bg-brand-600 px-3 py-1 text-sm text-white disabled:opacity-50">{saving ? '保存中…' : '保存标注'}</button><button onClick={() => onReorder(index - 1)} disabled={index === 0 || isDragging} className="rounded border px-3 py-1 text-sm disabled:opacity-50">上移</button><button onClick={() => onReorder(index + 1)} disabled={index === total - 1 || isDragging} className="rounded border px-3 py-1 text-sm disabled:opacity-50">下移</button><button onClick={onRemove} className="rounded border px-3 py-1 text-sm">移除</button></div>
    </div>
  </article>;
}

function ExportButton({ label, onClick, disabled }: { label: string; onClick: () => void; disabled: boolean }) { return <button onClick={onClick} disabled={disabled} className="rounded-lg border px-3 py-2 text-sm disabled:opacity-50">导出 {label}</button>; }
