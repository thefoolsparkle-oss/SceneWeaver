import { describe, expect, it } from 'vitest';
import { assetDateInfo, formatBytes, formatDuration, formatTimecode } from './mediaFormat';

describe('media formatting', () => {
  it('formats video durations for the media grid', () => {
    expect(formatDuration(65_400)).toBe('1:05');
    expect(formatDuration(null)).toBe('');
  });

  it('formats byte counts without overflowing its unit list', () => {
    expect(formatBytes(0)).toBe('0 B');
    expect(formatBytes(1_048_576)).toBe('1.0 MB');
    expect(formatBytes(1024 ** 6)).toBe('1048576.0 TB');
  });
});

it('formats exact timecodes', () => {
  expect(formatTimecode(3_661_234)).toBe('01:01:01.234');
});

it('uses trusted capture time before a filesystem modification time', () => {
  expect(assetDateInfo({ capture_time: 2_000, modified_at: 1_000 })).toEqual({ timestamp: 2_000, source: '拍摄/创建' });
  expect(assetDateInfo({ capture_time: null, modified_at: 1_000 })).toEqual({ timestamp: 1_000, source: '文件修改' });
});
