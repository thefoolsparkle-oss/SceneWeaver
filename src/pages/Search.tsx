import { useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import { open } from '@tauri-apps/plugin-dialog';
import { findAssetsForEntity, findSimilarAssets, findSimilarByReferenceImage, listEntities, recentSearches, searchAssets } from '@/api';
import { MediaGrid } from '@/components/MediaGrid';
import { SegmentPanel } from '@/components/SegmentPanel';
import { parseQueryConditions, replaceQueryCondition } from '@/lib/queryConditions';
import { ACG_CREATOR_PRESETS, applyAcgCreatorPack } from '@/lib/acgCreatorPack';
import { acgCreatorPackEnabled } from '@/api';
import { useQuery } from '@tanstack/react-query';
import type { MediaType, SearchRequest } from '@/types';

export default function Search() {
  const [query, setQuery] = useState('');
  const search = useMutation({ mutationFn: searchAssets });
  const similar = useMutation({ mutationFn: findSimilarAssets });
  const referenceImage = useMutation({ mutationFn: findSimilarByReferenceImage });
  const entityMatches = useMutation({ mutationFn: findAssetsForEntity });
  const [similarReference, setSimilarReference] = useState<string | null>(null);
  const [conditions, setConditions] = useState<SearchRequest | null>(null);
  const [mediaTypes, setMediaTypes] = useState<MediaType[]>([]);
  const [minQuality, setMinQuality] = useState<number | null>(null);
  const [entityId, setEntityId] = useState('');
  const [segmentFocus, setSegmentFocus] = useState<{ assetId: string; matchingSegmentIds?: string[] } | null>(null);
  const history = useQuery({ queryKey: ['recentSearches'], queryFn: recentSearches });
  const entities = useQuery({ queryKey: ['entities'], queryFn: listEntities });
  const acgPack = useQuery({ queryKey: ['acgCreatorPack'], queryFn: acgCreatorPackEnabled });

  const runQuery = (rawQuery: string) => {
    if (rawQuery.trim()) {
      const parsed = parseQueryConditions(rawQuery.trim());
      similar.reset(); entityMatches.reset(); setSimilarReference(null); setSegmentFocus(null);
      const request = { ...(acgPack.data ? applyAcgCreatorPack(parsed) : parsed), media_types: mediaTypes, min_quality_score: minQuality };
      setConditions(request);
      search.mutate(request);
    }
  };
  const submit = (event: React.FormEvent) => { event.preventDefault(); runQuery(query); };
  const chooseReferenceImage = async () => {
    const path = await open({ multiple: false, filters: [{ name: '图片', extensions: ['jpg', 'jpeg', 'png', 'webp', 'gif', 'bmp'] }] });
    if (typeof path === 'string') { setSimilarReference(path); search.reset(); similar.reset(); entityMatches.reset(); setSegmentFocus(null); referenceImage.mutate(path); }
  };
  const searchEntity = () => {
    if (!entityId) return;
    search.reset(); similar.reset(); referenceImage.reset(); setSimilarReference(null); setSegmentFocus(null);
    entityMatches.mutate(entityId);
  };
  const keywordAssets = search.data?.map((result) => result.asset) ?? [];
  const keywordExplanations = Object.fromEntries((search.data ?? []).map((result) => [result.asset.id, { reasons: result.match_reasons, unmet: result.unmet_should }]));
  const keywordMatchingSegments = Object.fromEntries((search.data ?? []).map((result) => [result.asset.id, result.matching_segment_ids]));
  const activeAssets = entityMatches.data ?? referenceImage.data ?? similar.data ?? keywordAssets;
  const removeCondition = (kind: 'must' | 'should' | 'must_not', term: string) => {
    if (!conditions) return;
    const request = { ...conditions, [kind]: conditions[kind].filter((value) => value !== term) };
    setConditions(request); search.mutate(request);
  };
  const editCondition = (kind: 'must' | 'should' | 'must_not', term: string, replacement: string) => {
    if (!conditions || !replacement.trim() || replacement.trim() === term) return;
    const request = replaceQueryCondition(conditions, kind, term, replacement);
    setConditions(request); search.mutate(request);
  };
  const toggleMediaType = (mediaType: MediaType) => setMediaTypes((current) => current.includes(mediaType) ? current.filter((value) => value !== mediaType) : [...current, mediaType]);

  return (
    <div className="p-8">
      <h1 className="mb-2 text-2xl font-bold">搜索素材</h1>
      <p className="mb-5 text-sm text-neutral-500">可结合关键词、参考图和本地实体搜索；安装本地语义模型后会额外使用图文向量召回。{acgPack.data ? ' ACG Creator Pack 已启用，会按本地词典增加优先匹配。' : ''}</p>
      <form onSubmit={submit} className="mb-6 flex gap-2">
        <input
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder="例如：雨夜、角色、采访"
          className="min-w-0 flex-1 rounded-lg border border-neutral-300 px-4 py-2.5 dark:border-neutral-700 dark:bg-neutral-900"
        />
        <button disabled={!query.trim() || search.isPending} className="rounded-lg bg-brand-600 px-5 font-medium text-white disabled:opacity-50">
          {search.isPending ? '搜索中…' : '搜索'}
        </button>
        <button type="button" onClick={chooseReferenceImage} disabled={referenceImage.isPending} className="rounded-lg border px-4 text-sm disabled:opacity-50">{referenceImage.isPending ? '检索中…' : '参考图'}</button>
      </form>
      {acgPack.data && <div className="mb-4 flex flex-wrap items-center gap-2"><span className="text-xs text-neutral-500">ACG 预设：</span>{ACG_CREATOR_PRESETS.map((preset) => <button key={preset.id} type="button" onClick={() => { setQuery(preset.query); runQuery(preset.query); }} className="rounded-full border border-violet-200 bg-violet-50 px-2 py-1 text-xs text-violet-800 hover:bg-violet-100 dark:border-violet-800 dark:bg-violet-950 dark:text-violet-200">{preset.label}</button>)}</div>}
      <div className="mb-4 flex flex-wrap items-center gap-2">
        <label className="text-xs text-neutral-500" htmlFor="entity-search">实体：</label>
        <select id="entity-search" value={entityId} onChange={(event) => setEntityId(event.target.value)} className="rounded border px-2 py-1.5 text-sm dark:border-neutral-700 dark:bg-neutral-900">
          <option value="">选择已创建的实体</option>
          {entities.data?.map((entity) => <option key={entity.id} value={entity.id}>{entity.name}（{entity.entity_type}）</option>)}
        </select>
        <button type="button" onClick={searchEntity} disabled={!entityId || entityMatches.isPending} className="rounded border px-3 py-1.5 text-sm disabled:opacity-50">{entityMatches.isPending ? '查找中…' : '查找实体素材'}</button>
      </div>
      <div className="mb-4 flex flex-wrap items-center gap-2 text-xs text-neutral-500"><span>素材类型：</span>{([{ value: 'image', label: '图片' }, { value: 'video', label: '视频' }, { value: 'audio', label: '音频' }] as const).map(({ value, label }) => <button type="button" key={value} onClick={() => toggleMediaType(value)} className={`rounded-full border px-2 py-1 ${mediaTypes.includes(value) ? 'border-brand-500 bg-brand-50 text-brand-700' : ''}`}>{label}</button>)}</div>
      <label className="mb-4 flex items-center gap-2 text-xs text-neutral-500">最低视频片段质量<select value={minQuality ?? ''} onChange={(event) => setMinQuality(event.target.value === '' ? null : Number(event.target.value))} className="rounded border px-2 py-1"><option value="">不限</option><option value="0.5">50%</option><option value="0.7">70%</option><option value="0.85">85%</option></select></label>
      {!!history.data?.length && <div className="mb-4 flex flex-wrap items-center gap-2 text-sm text-neutral-500"><span>最近搜索：</span>{history.data.map((item) => <button key={item} onClick={() => setQuery(item)} className="rounded-full border px-2 py-1 text-xs hover:bg-neutral-100 dark:hover:bg-neutral-800">{item}</button>)}</div>}
      {conditions && (
        <div className="mb-4 flex flex-wrap gap-2 text-xs">
          {conditions.must.map((term) => <ConditionChip key={`must-${term}`} prefix="必须" term={term} className="bg-brand-100 text-brand-700" onChange={(replacement) => editCondition('must', term, replacement)} onRemove={() => removeCondition('must', term)} />)}
          {conditions.should.map((term) => <ConditionChip key={`should-${term}`} prefix="优先" term={term} className="bg-amber-100 text-amber-700" onChange={(replacement) => editCondition('should', term, replacement)} onRemove={() => removeCondition('should', term)} />)}
          {conditions.must_not.map((term) => <ConditionChip key={`not-${term}`} prefix="排除" term={term} className="bg-red-100 text-red-700" onChange={(replacement) => editCondition('must_not', term, replacement)} onRemove={() => removeCondition('must_not', term)} />)}
        </div>
      )}
      {(search.isError || similar.isError || referenceImage.isError || entityMatches.isError) && <p className="mb-4 text-sm text-red-600">搜索失败：{(entityMatches.error ?? referenceImage.error ?? similar.error ?? search.error)?.message}</p>}
      {(search.data || similar.data || referenceImage.data || entityMatches.data) && <p className="mb-4 text-sm text-neutral-500">{entityMatches.data ? '实体匹配结果' : referenceImage.data ? '参考图相似结果' : similarReference ? '视觉相似结果' : '关键词结果'}：{activeAssets.length} 项</p>}
      {(search.data || similar.data || referenceImage.data || entityMatches.data) && <MediaGrid assets={activeAssets} explanations={referenceImage.data || similar.data || entityMatches.data ? undefined : keywordExplanations} matchingSegments={referenceImage.data || similar.data || entityMatches.data ? undefined : keywordMatchingSegments} onViewSegments={(assetId, matchingSegmentIds) => setSegmentFocus({ assetId, matchingSegmentIds })} onFindSimilar={(assetId) => { setSimilarReference(assetId); referenceImage.reset(); entityMatches.reset(); setSegmentFocus(null); similar.mutate(assetId); }} />}
      {segmentFocus && <SegmentPanel assetId={segmentFocus.assetId} matchingSegmentIds={segmentFocus.matchingSegmentIds} onClose={() => setSegmentFocus(null)} />}
    </div>
  );
}

function ConditionChip({ prefix, term, className, onChange, onRemove }: { prefix: string; term: string; className: string; onChange: (value: string) => void; onRemove: () => void }) {
  const [editing, setEditing] = useState(false);
  const [value, setValue] = useState(term);
  const commit = () => { setEditing(false); onChange(value); };
  if (editing) return <form onSubmit={(event) => { event.preventDefault(); commit(); }} className={`rounded-full px-2 py-1 ${className}`}><input autoFocus value={value} onChange={(event) => setValue(event.target.value)} onBlur={commit} aria-label={`${prefix} 条件`} className="w-20 bg-transparent outline-none" /></form>;
  return <span className={`inline-flex items-center rounded-full px-2 py-1 ${className}`}><button type="button" onClick={() => setEditing(true)} title="编辑条件" className="text-left">{prefix}：{term}</button><button type="button" onClick={onRemove} title="移除此条件并重新搜索" className="ml-1">×</button></span>;
}
