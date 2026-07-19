import { describe, expect, it } from 'vitest';
import { formatBytes, formatDuration, formatTimecode } from './mediaFormat';

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
