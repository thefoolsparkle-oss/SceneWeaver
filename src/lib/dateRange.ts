export function dateInputBoundary(value: string, boundary: 'start' | 'end'): number | null {
  if (!value) return null;
  const date = new Date(`${value}T${boundary === 'start' ? '00:00:00.000' : '23:59:59.999'}`);
  const timestamp = date.getTime();
  return Number.isNaN(timestamp) ? null : timestamp;
}

export function assetMatchesDateRange(
  asset: { capture_time: number | null; modified_at: number },
  dateStartMs: number | null,
  dateEndMs: number | null,
): boolean {
  const timestamp = asset.capture_time ?? asset.modified_at;
  return (dateStartMs === null || timestamp >= dateStartMs)
    && (dateEndMs === null || timestamp <= dateEndMs);
}
