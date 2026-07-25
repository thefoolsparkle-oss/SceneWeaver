import { describe, expect, it } from 'vitest';
import { assetMatchesDateRange, dateInputBoundary } from './dateRange';

describe('local date ranges', () => {
  it('creates inclusive local-day boundaries', () => {
    const start = dateInputBoundary('2026-07-25', 'start');
    const end = dateInputBoundary('2026-07-25', 'end');
    expect(start).not.toBeNull();
    expect(end).not.toBeNull();
    expect(end! - start!).toBe(86_399_999);
  });

  it('uses capture time before modification time and respects inclusive bounds', () => {
    expect(assetMatchesDateRange({ capture_time: 2_000, modified_at: 1_000 }, 2_000, 2_000)).toBe(true);
    expect(assetMatchesDateRange({ capture_time: 2_000, modified_at: 1_000 }, null, 1_999)).toBe(false);
    expect(assetMatchesDateRange({ capture_time: null, modified_at: 1_000 }, 1_000, 1_000)).toBe(true);
  });
});
