import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { open } from '@tauri-apps/plugin-dialog';
import {
  addEntityReferenceImage,
  createEntity,
  findAssetsForEntity,
  listEntities,
  listEntityReferences,
  removeEntityReference,
  setEntityAssetFeedback,
} from '@/api';
import { MediaGrid } from '@/components/MediaGrid';

export default function Entities() {
  const queryClient = useQueryClient(); const [name, setName] = useState(''); const [type, setType] = useState('character'); const [aliases, setAliases] = useState('');
  const entities = useQuery({ queryKey: ['entities'], queryFn: listEntities });
  const create = useMutation({ mutationFn: createEntity, onSuccess: () => { setName(''); setAliases(''); queryClient.invalidateQueries({ queryKey: ['entities'] }); } });
  return <div className="p-8"><h1 className="mb-2 text-2xl font-bold">实体</h1><p className="mb-6 text-sm text-neutral-500">创建角色、人物、产品或自定义视觉主体；数据只保存在本机。</p>
    <form onSubmit={(event) => { event.preventDefault(); if (name.trim()) create.mutate({ name, entity_type: type, aliases: aliases.split(/[，,]/).map((value) => value.trim()).filter(Boolean) }); }} className="mb-6 flex flex-wrap gap-2 rounded-xl border p-4">
      <select value={type} onChange={(event) => setType(event.target.value)} className="rounded border px-2 py-2"><option value="character">角色</option><option value="person">人物</option><option value="product">产品</option><option value="custom">自定义</option></select><input value={name} onChange={(event) => setName(event.target.value)} placeholder="实体名称" className="rounded border px-3 py-2"/><input value={aliases} onChange={(event) => setAliases(event.target.value)} placeholder="别名，用逗号分隔" className="flex-1 rounded border px-3 py-2"/><button className="rounded bg-brand-600 px-4 text-white">创建</button>
    </form>{create.isError && <p className="text-red-600">创建失败：{create.error.message}</p>}<div className="grid gap-3 sm:grid-cols-2">{entities.data?.map((entity) => <EntityCard key={entity.id} entity={entity} />)}</div>
  </div>;
}

function EntityCard({ entity }: { entity: Awaited<ReturnType<typeof listEntities>>[number] }) {
  const references = useQuery({ queryKey: ['entityReferences', entity.id], queryFn: () => listEntityReferences(entity.id) });
  const reference = useMutation({ mutationFn: ({ imagePath, isPositive }: { imagePath: string; isPositive: boolean }) => addEntityReferenceImage(entity.id, imagePath, isPositive), onSuccess: () => references.refetch() });
  const removeReference = useMutation({ mutationFn: (referenceId: string) => removeEntityReference(entity.id, referenceId), onSuccess: () => references.refetch() });
  const feedback = useMutation({ mutationFn: ({ assetId, isPositive }: { assetId: string; isPositive: boolean }) => setEntityAssetFeedback(entity.id, assetId, isPositive), onSuccess: () => references.refetch() });
  const matches = useMutation({ mutationFn: () => findAssetsForEntity(entity.id) });
  const chooseReference = async (isPositive: boolean) => { const path = await open({ multiple: false, filters: [{ name: '图片', extensions: ['jpg', 'jpeg', 'png', 'webp', 'gif', 'bmp'] }] }); if (typeof path === 'string') reference.mutate({ imagePath: path, isPositive }); };
  return <section className="rounded-xl border bg-white p-4 dark:bg-neutral-950"><p className="font-medium">{entity.name}</p><p className="text-xs text-neutral-500">{entity.entity_type}</p>{entity.aliases.length > 0 && <p className="mt-2 text-sm text-neutral-600">别名：{entity.aliases.join('、')}</p>}<p className="mt-2 text-xs text-neutral-500">正参考 {references.data?.filter((item) => item.is_positive).length ?? 0} 张 · 负参考 {references.data?.filter((item) => !item.is_positive).length ?? 0} 张</p><div className="mt-3 flex flex-wrap gap-2"><button onClick={() => chooseReference(true)} disabled={reference.isPending} className="rounded border px-2 py-1 text-xs">添加正参考图</button><button onClick={() => chooseReference(false)} disabled={reference.isPending} className="rounded border px-2 py-1 text-xs">添加负参考图</button><button onClick={() => matches.mutate()} disabled={matches.isPending} className="rounded bg-brand-600 px-2 py-1 text-xs text-white disabled:opacity-50">查找实体素材</button></div>{references.data && references.data.length > 0 && <ul className="mt-3 space-y-1 text-xs">{references.data.map((item) => <li key={item.id} className="flex items-center gap-2 rounded bg-neutral-50 px-2 py-1 dark:bg-neutral-900"><span className={item.is_positive ? 'text-emerald-700 dark:text-emerald-400' : 'text-rose-700 dark:text-rose-400'}>{item.is_positive ? '正参考' : '负参考'}</span><span className="min-w-0 flex-1 truncate" title={item.image_path ?? item.asset_id ?? ''}>{item.image_path ?? (item.asset_id ? `素材反馈：${item.asset_id}` : '无路径')}</span><button onClick={() => removeReference.mutate(item.id)} disabled={removeReference.isPending} className="rounded border px-1.5 py-0.5 hover:bg-white disabled:opacity-50 dark:hover:bg-neutral-800">移除</button></li>)}</ul>}{reference.isError && <p className="mt-2 text-xs text-red-600">{reference.error.message}</p>}{removeReference.isError && <p className="mt-2 text-xs text-red-600">移除失败：{removeReference.error.message}</p>}{feedback.isError && <p className="mt-2 text-xs text-red-600">反馈保存失败：{feedback.error.message}</p>}{matches.isError && <p className="mt-2 text-xs text-red-600">{matches.error.message}</p>}{matches.data && <div className="mt-4"><p className="mb-2 text-xs text-neutral-500">实体匹配素材 {matches.data.length} 项；将鼠标移到卡片上可标记“是实体”或“非实体”。</p><MediaGrid assets={matches.data} onEntityFeedback={(assetId, isPositive) => feedback.mutate({ assetId, isPositive })} /></div>}</section>;
}
