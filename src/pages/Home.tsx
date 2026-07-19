import { useQuery } from '@tanstack/react-query';
import { Link } from 'react-router-dom';
import { getAppStats } from '@/api';

export default function Home() {
  const { data: stats, isLoading } = useQuery({
    queryKey: ['appStats'],
    queryFn: getAppStats,
  });

  return (
    <div className="p-8">
      <div className="mx-auto max-w-4xl">
        <h1 className="mb-2 text-3xl font-bold">说出你要的镜头，直接回到创作。</h1>
        <p className="mb-8 text-neutral-600 dark:text-neutral-400">
          SceneWeaver 帮助你用自然语言、参考图和组合条件快速找到本地图片与视频素材。
        </p>

        <div className="mb-8 grid grid-cols-2 gap-4 sm:grid-cols-4">
          <StatCard label="素材库" value={isLoading ? '-' : stats?.library_count ?? 0} />
          <StatCard label="素材总数" value={isLoading ? '-' : stats?.asset_count ?? 0} />
          <StatCard label="图片" value={isLoading ? '-' : stats?.image_count ?? 0} />
          <StatCard label="视频" value={isLoading ? '-' : stats?.video_count ?? 0} />
        </div>

        <div className="flex gap-3">
          <Link
            to="/libraries"
            className="rounded-lg bg-brand-600 px-5 py-2.5 font-medium text-white hover:bg-brand-700"
          >
            添加素材库
          </Link>
          <Link
            to="/settings"
            className="rounded-lg border border-neutral-300 bg-white px-5 py-2.5 font-medium text-neutral-700 hover:bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-200"
          >
            打开设置
          </Link>
        </div>
      </div>
    </div>
  );
}

function StatCard({ label, value }: { label: string; value: number | string }) {
  return (
    <div className="rounded-xl border border-neutral-200 bg-white p-4 dark:border-neutral-800 dark:bg-neutral-950">
      <div className="text-2xl font-bold">{value}</div>
      <div className="text-sm text-neutral-500">{label}</div>
    </div>
  );
}
