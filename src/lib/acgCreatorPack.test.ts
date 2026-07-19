import { describe, expect, it } from 'vitest';
import { ACG_CREATOR_PRESETS, applyAcgCreatorPack } from './acgCreatorPack';

describe('ACG Creator Pack', () => {
  it('adds local filename synonyms as soft conditions', () => {
    expect(applyAcgCreatorPack({ raw_query: '角色 战斗', must: ['角色', '战斗'], should: [], must_not: [] }).should).toEqual(['battle', 'fight', 'combat']);
  });

  it('ships editable local ACG search presets', () => {
    expect(ACG_CREATOR_PRESETS.map((preset) => preset.id)).toContain('battle-clean');
    expect(ACG_CREATOR_PRESETS.find((preset) => preset.id === 'battle-clean')?.query).toContain('不要');
  });

  it('keeps ACG exclusion synonyms as hard exclusions', () => {
    expect(applyAcgCreatorPack({ raw_query: '不要游戏 UI', must: [], should: [], must_not: ['游戏 UI'] }).must_not).toEqual(['游戏 UI', 'ui', 'hud', 'interface']);
  });
});
