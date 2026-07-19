export interface ScanMetrics {
  processed: number;
  total: number;
  errors: number;
}

const emptyMetrics: ScanMetrics = { processed: 0, total: 0, errors: 0 };

export function scanMetrics(checkpoint: string | null): ScanMetrics {
  if (!checkpoint) return emptyMetrics;
  try {
    const value: unknown = JSON.parse(checkpoint);
    if (
      typeof value === 'object' &&
      value !== null &&
      typeof (value as ScanMetrics).processed === 'number' &&
      typeof (value as ScanMetrics).total === 'number' &&
      typeof (value as ScanMetrics).errors === 'number'
    ) {
      return value as ScanMetrics;
    }
  } catch {
    // A future job type may have a different checkpoint format.
  }
  return emptyMetrics;
}
