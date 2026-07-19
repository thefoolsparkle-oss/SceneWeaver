import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { save } from '@tauri-apps/plugin-dialog';
import { acgCreatorPackEnabled, cacheSize, clearMediaCache, exportDatabaseSnapshot, installSemanticModel, reindexSemanticAssets, semanticModelStatus, setAcgCreatorPackEnabled } from '@/api';

export default function Settings() {
  const [darkMode, setDarkMode] = useState(false);
  const queryClient = useQueryClient();
  const acgPack = useQuery({ queryKey: ['acgCreatorPack'], queryFn: acgCreatorPackEnabled });
  const setAcgPack = useMutation({ mutationFn: setAcgCreatorPackEnabled, onSuccess: () => queryClient.invalidateQueries({ queryKey: ['acgCreatorPack'] }) });
  const semanticStatus = useQuery({ queryKey: ['semanticModelStatus'], queryFn: semanticModelStatus });
  const installSemantic = useMutation({
    mutationFn: installSemanticModel,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['semanticModelStatus'] }),
  });
  const semanticReindex = useMutation({ mutationFn: reindexSemanticAssets });
  const cache = useQuery({ queryKey: ['cacheSize'], queryFn: cacheSize });
  const clearCache = useMutation({ mutationFn: clearMediaCache, onSuccess: () => queryClient.invalidateQueries({ queryKey: ['cacheSize'] }) });
  const exportDatabase = async () => { const path = await save({ defaultPath: 'sceneweaver-backup.db', filters: [{ name: 'SQLite database', extensions: ['db'] }] }); if (path) await exportDatabaseSnapshot(path); };

  const toggleDarkMode = () => {
    setDarkMode(!darkMode);
    document.documentElement.classList.toggle('dark');
  };

  return (
    <div className="p-8">
      <h1 className="mb-6 text-2xl font-bold">设置</h1>
      <div className="max-w-xl space-y-4 rounded-xl border border-neutral-200 bg-white p-5 dark:border-neutral-800 dark:bg-neutral-950">
        <div className="flex items-center justify-between">
          <div>
            <div className="font-medium">深色模式</div>
            <div className="text-sm text-neutral-500">切换界面深色/浅色主题</div>
          </div>
          <button
            onClick={toggleDarkMode}
            className={`relative h-6 w-11 rounded-full transition-colors ${darkMode ? 'bg-brand-600' : 'bg-neutral-300'}`}
          >
            <span
              className={`absolute left-1 top-1 h-4 w-4 rounded-full bg-white transition-transform ${darkMode ? 'translate-x-5' : ''}`}
            />
          </button>
        </div>
        <div className="flex items-center justify-between border-t pt-4"><div><div className="font-medium">导出本地数据库</div><div className="text-sm text-neutral-500">创建一致的 SQLite 快照，不包含原始媒体或缓存。</div></div><button onClick={() => void exportDatabase()} className="rounded-lg border border-neutral-300 px-3 py-1.5 text-sm hover:bg-neutral-50 dark:border-neutral-700 dark:hover:bg-neutral-800">导出 .db</button></div>

        <div className="flex items-center justify-between gap-4 border-t pt-4">
          <div>
            <div className="font-medium">ACG Creator Pack</div>
            <div className="text-sm text-neutral-500">启用本地 ACG 查询词典：战斗、剧情、过场、Gameplay、菜单、UI、镜头语言。</div>
          </div>
          <button
            aria-label="切换 ACG Creator Pack"
            onClick={() => setAcgPack.mutate(!(acgPack.data ?? false))}
            disabled={acgPack.isLoading || setAcgPack.isPending}
            className={`relative h-6 w-11 shrink-0 rounded-full transition-colors disabled:opacity-50 ${acgPack.data ? 'bg-brand-600' : 'bg-neutral-300'}`}
          >
            <span className={`absolute left-1 top-1 h-4 w-4 rounded-full bg-white transition-transform ${acgPack.data ? 'translate-x-5' : ''}`} />
          </button>
        </div>
        {setAcgPack.isError && <p className="text-sm text-red-600">ACG Creator Pack 设置保存失败：{setAcgPack.error.message}</p>}

        <div className="border-t pt-4">
          <div className="flex items-start justify-between gap-4">
            <div>
              <div className="font-medium">本地语义模型（实验性）</div>
              <div className="text-sm text-neutral-500">仅在点击下载后联网，模型和向量均保存在本机；未安装时仍使用关键词与颜色检索。</div>
              <div className="mt-1 text-xs text-neutral-500">{semanticStatus.data?.message ?? '正在检测本地模型状态…'}</div>
            </div>
            <div className="flex shrink-0 gap-2">
              {!semanticStatus.data?.modelInstalled && (
                <button
                  onClick={() => installSemantic.mutate()}
                  disabled={installSemantic.isPending || !semanticStatus.data?.runtimeAvailable}
                  className="rounded-lg bg-brand-600 px-3 py-1.5 text-sm text-white hover:bg-brand-700 disabled:opacity-50"
                >{installSemantic.isPending ? '正在下载…' : '下载模型'}</button>
              )}
              {semanticStatus.data?.ready && (
                <button
                  onClick={() => semanticReindex.mutate()}
                  disabled={semanticReindex.isPending}
                  className="rounded-lg border border-neutral-300 px-3 py-1.5 text-sm hover:bg-neutral-50 disabled:opacity-50 dark:border-neutral-700 dark:hover:bg-neutral-800"
                >{semanticReindex.isPending ? '建立索引…' : '建立语义索引'}</button>
              )}
            </div>
          </div>
          {installSemantic.isError && <p className="mt-2 text-sm text-red-600">模型下载失败：{installSemantic.error.message}</p>}
          {semanticReindex.isError && <p className="mt-2 text-sm text-red-600">语义索引失败：{semanticReindex.error.message}</p>}
          {semanticReindex.data && <p className="mt-2 text-sm text-green-700">素材：已索引 {semanticReindex.data.indexed} 项，跳过 {semanticReindex.data.skipped} 项，失败 {semanticReindex.data.failed} 项。实体参考：已索引 {semanticReindex.data.entityReferencesIndexed} 项，跳过 {semanticReindex.data.entityReferencesSkipped} 项，失败 {semanticReindex.data.entityReferencesFailed} 项。</p>}
        </div>

        <div className="flex items-center justify-between">
          <div>
            <div className="font-medium">云端增强</div>
            <div className="text-sm text-neutral-500">默认关闭，开启前将提示上传内容</div>
          </div>
          <span className="text-sm text-neutral-500">后续版本支持</span>
        </div>

        <div className="flex items-center justify-between">
          <div>
            <div className="font-medium">缓存管理</div>
            <div className="text-sm text-neutral-500">查看与清理本地缩略图和代理文件</div>
          </div>
          <button onClick={() => clearCache.mutate()} disabled={clearCache.isPending} className="rounded-lg border border-neutral-300 px-3 py-1.5 text-sm hover:bg-neutral-50 disabled:opacity-50 dark:border-neutral-700 dark:hover:bg-neutral-800">
            {clearCache.isPending ? '清理中…' : `清理 ${Math.round((cache.data ?? 0) / 1024)} KB`}
          </button>
        </div>
      </div>
    </div>
  );
}
