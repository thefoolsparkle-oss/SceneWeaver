export function formatDuration(milliseconds: number | null): string {
  if (milliseconds === null) return '';
  const totalSeconds = Math.round(milliseconds / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${seconds.toString().padStart(2, '0')}`;
}

export function formatTimecode(milliseconds: number): string {
  const value = Math.max(0, Math.floor(milliseconds));
  const hours = Math.floor(value / 3_600_000);
  const minutes = Math.floor(value / 60_000) % 60;
  const seconds = Math.floor(value / 1_000) % 60;
  const fraction = value % 1_000;
  return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}.${fraction.toString().padStart(3, '0')}`;
}

export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const unit = 1024;
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const index = Math.min(Math.floor(Math.log(bytes) / Math.log(unit)), units.length - 1);
  return `${(bytes / unit ** index).toFixed(1)} ${units[index]}`;
}

export function assetDateInfo(asset: { capture_time: number | null; modified_at: number }): { timestamp: number; source: '拍摄/创建' | '文件修改' } {
  if (asset.capture_time !== null) return { timestamp: asset.capture_time, source: '拍摄/创建' };
  return { timestamp: asset.modified_at, source: '文件修改' };
}

export function formatAssetDate(asset: { capture_time: number | null; modified_at: number }): string {
  const { timestamp, source } = assetDateInfo(asset);
  const formatted = new Intl.DateTimeFormat('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  }).format(new Date(timestamp));
  return `${source}：${formatted}`;
}
