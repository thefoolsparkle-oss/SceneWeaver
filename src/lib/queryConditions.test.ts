import { describe, expect, it } from 'vitest';
import { parseQueryConditions, replaceQueryCondition } from './queryConditions';

describe('query condition parsing', () => {
  it('extracts Chinese negative and preferred conditions', () => {
    expect(parseQueryConditions('雨夜 角色，不要字幕、战斗和 UI，优先侧脸')).toMatchObject({
      must: ['雨夜', '角色'], should: ['侧脸'], must_not: ['字幕', '战斗', 'UI'],
    });
  });
});

describe('query condition editing', () => {
  it('updates only the requested structured condition', () => {
    const parsed = parseQueryConditions('雨夜 优先角色 不要字幕');
    expect(replaceQueryCondition(parsed, 'should', '角色', '侧脸')).toMatchObject({
      must: ['雨夜'], should: ['侧脸'], must_not: ['字幕'],
    });
  });
});
