import { describe, expect, it } from 'vitest';
import { scanMetrics } from './scanMetrics';

describe('scan metrics', () => {
  it('reads a persisted scan checkpoint', () => {
    expect(scanMetrics('{"processed":4,"total":10,"errors":1}')).toEqual({
      processed: 4,
      total: 10,
      errors: 1,
    });
  });

  it('safely handles absent or unrelated checkpoints', () => {
    expect(scanMetrics(null)).toEqual({ processed: 0, total: 0, errors: 0 });
    expect(scanMetrics('{"cursor":12}')).toEqual({ processed: 0, total: 0, errors: 0 });
  });
});
