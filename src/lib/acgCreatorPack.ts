import type { SearchRequest } from '@/types';

const ACG_SYNONYMS: Record<string, string[]> = {
  战斗: ['battle', 'fight', 'combat'],
  剧情: ['story', 'cutscene', 'event'],
  过场: ['cutscene', 'cinematic'],
  菜单: ['menu', 'pause'],
  UI: ['ui', 'hud', 'interface'],
  '游戏 UI': ['ui', 'hud', 'interface'],
  Gameplay: ['gameplay', 'play'],
  正脸: ['front', 'face'],
  侧脸: ['profile', 'side'],
  背影: ['back', 'rear'],
};

export const ACG_CREATOR_PRESETS = [
  { id: 'character-closeup', label: '角色近景', query: '角色 近景' },
  { id: 'rainy-profile', label: '雨夜侧脸', query: '角色 雨夜 侧脸' },
  { id: 'battle-clean', label: '战斗无 UI/字幕', query: '战斗 不要 游戏 UI、字幕' },
  { id: 'story-cutscene', label: '剧情过场', query: '剧情 过场 优先 近景' },
  { id: 'ending-emotion', label: '情绪结尾', query: '角色 优先 回头、侧脸 不要 战斗、游戏 UI' },
] as const;

export function applyAcgCreatorPack(request: SearchRequest): SearchRequest {
  const positiveTerms = [...request.must, ...request.should];
  const positiveSynonyms = positiveTerms.flatMap((term) => ACG_SYNONYMS[term] ?? []);
  const excludedSynonyms = request.must_not.flatMap((term) => ACG_SYNONYMS[term] ?? []);
  return {
    ...request,
    should: [...new Set([...request.should, ...positiveSynonyms])],
    must_not: [...new Set([...request.must_not, ...excludedSynonyms])],
  };
}
