import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { open } from '@tauri-apps/plugin-dialog';
import { Link } from 'react-router-dom';
import { createLibrary, listLibraries, startScan } from '@/api';
import type { Library } from '@/types';

export default function Libraries() {
  const queryClient = useQueryClient();
  const { data: libraries, isLoading } = useQuery({
    queryKey: ['libraries'],
    queryFn: listLibraries,
  });
  const [name, setName] = useState('');
  const [path, setPath] = useState('');

  const createMutation = useMutation({
    mutationFn: createLibrary,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['libraries'] });
      setName('');
      setPath('');
    },
  });

  const scanMutation = useMutation({
    mutationFn: startScan,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['libraries'] });
      queryClient.invalidateQueries({ queryKey: ['jobs'] });
    },
  });

  const pickFolder = async () => {
    const selected = await open({ directory: true, multiple: false });
    if (selected && typeof selected === 'string') {
      setPath(selected);
      if (!name) {
        setName(selected.split(/[\\/]/).pop() ?? '');
      }
    }
  };

  const handleCreate = () => {
    if (!name.trim() || !path.trim()) return;
    createMutation.mutate({
      name: name.trim(),
      root_path: path.trim(),
      index_profile: 'balanced',
    });
  };

  return (
    <div className="p-8">
      <h1 className="mb-6 text-2xl font-bold">素材库</h1>

      <div className="mb-8 rounded-xl border border-neutral-200 bg-white p-5 dark:border-neutral-800 dark:bg-neutral-950">
        <h2 className="mb-4 font-semibold">添加素材库</h2>
        <div className="flex flex-col gap-3 sm:flex-row">
          <input
            type="text"
            placeholder="素材库名称"
            value={name}
            onChange={(e) => setName(e.target.value)}
            className="flex-1 rounded-lg border border-neutral-300 px-3 py-2 dark:border-neutral-700 dark:bg-neutral-900"
          />
          <div className="flex flex-1 gap-2">
            <input
              type="text"
              placeholder="选择文件夹"
              value={path}
              readOnly
              className="min-w-0 flex-1 rounded-lg border border-neutral-300 px-3 py-2 dark:border-neutral-700 dark:bg-neutral-900"
            />
            <button
              onClick={pickFolder}
              className="whitespace-nowrap rounded-lg border border-neutral-300 bg-white px-3 py-2 text-sm hover:bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-800"
            >
              选择
            </button>
          </div>
          <button
            onClick={handleCreate}
            disabled={createMutation.isPending || !name.trim() || !path.trim()}
            className="rounded-lg bg-brand-600 px-5 py-2 font-medium text-white hover:bg-brand-700 disabled:opacity-50"
          >
            {createMutation.isPending ? '创建中…' : '创建'}
          </button>
        </div>
        {createMutation.isError && (
          <p className="mt-3 text-sm text-red-600">
            创建失败：{createMutation.error?.message ?? '未知错误'}
          </p>
        )}
      </div>

      {isLoading ? (
        <p className="text-neutral-500">加载中…</p>
      ) : libraries?.length === 0 ? (
        <p className="text-neutral-500">暂无素材库，请添加一个文件夹。</p>
      ) : (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {libraries?.map((lib) => (
            <LibraryCard
              key={lib.id}
              library={lib}
              onScan={() => scanMutation.mutate(lib.id)}
              isScanning={scanMutation.isPending && scanMutation.variables === lib.id}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function LibraryCard({
  library,
  onScan,
  isScanning,
}: {
  library: Library;
  onScan: () => void;
  isScanning: boolean;
}) {
  return (
    <div className="rounded-xl border border-neutral-200 bg-white p-4 dark:border-neutral-800 dark:bg-neutral-950">
      <div className="mb-2 flex items-start justify-between">
        <Link to={`/libraries/${library.id}`} className="font-semibold hover:text-brand-600">
          {library.name}
        </Link>
        <StatusBadge status={library.status} />
      </div>
      <p className="mb-3 truncate text-sm text-neutral-500" title={library.root_path}>
        {library.root_path}
      </p>
      <div className="flex gap-2">
        <button
          onClick={onScan}
          disabled={isScanning || library.status === 'scanning'}
          className="rounded-lg bg-brand-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-brand-700 disabled:opacity-50"
        >
          {isScanning ? '启动中…' : '扫描'}
        </button>
        <Link
          to={`/libraries/${library.id}`}
          className="rounded-lg border border-neutral-300 px-3 py-1.5 text-sm hover:bg-neutral-50 dark:border-neutral-700 dark:hover:bg-neutral-800"
        >
          查看
        </Link>
      </div>
    </div>
  );
}

function StatusBadge({ status }: { status: Library['status'] }) {
  const map: Record<Library['status'], string> = {
    idle: '空闲',
    scanning: '扫描中',
    paused: '已暂停',
    error: '错误',
  };
  const color: Record<Library['status'], string> = {
    idle: 'bg-neutral-100 text-neutral-700 dark:bg-neutral-800 dark:text-neutral-300',
    scanning: 'bg-brand-100 text-brand-700 dark:bg-brand-900 dark:text-brand-100',
    paused: 'bg-amber-100 text-amber-700 dark:bg-amber-900 dark:text-amber-100',
    error: 'bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-100',
  };
  return (
    <span className={`rounded-full px-2 py-0.5 text-xs font-medium ${color[status]}`}>
      {map[status]}
    </span>
  );
}
