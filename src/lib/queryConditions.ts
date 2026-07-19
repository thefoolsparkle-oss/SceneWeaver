import type { SearchRequest } from '@/types';

export function parseQueryConditions(rawQuery: string): SearchRequest {
  const mustNot: string[] = [];
  const should: string[] = [];
  const splitTerms = (value: string) => value.split(/[，,、/]|和|及|与/g).map((term) => term.trim()).filter(Boolean);
  let cleaned = rawQuery.replace(/(?:不要|不含|排除)\s*([\s\S]*?)(?=(?:不要|不含|排除|优先|最好|尽量)|[。；;]|$)/g, (_, terms: string) => {
    mustNot.push(...splitTerms(terms));
    return '';
  });
  cleaned = cleaned.replace(/(?:优先|最好|尽量)\s*([\s\S]*?)(?=(?:不要|不含|排除|优先|最好|尽量)|[。；;]|$)/g, (_, terms: string) => {
    should.push(...splitTerms(terms));
    return '';
  });
  const must = cleaned.split(/[，,。；;\s]+/).map((term) => term.trim()).filter(Boolean);
  return { raw_query: rawQuery, must, should, must_not: mustNot, media_types: [], min_quality_score: null };
}

export function replaceQueryCondition(request: SearchRequest, kind: 'must' | 'should' | 'must_not', term: string, replacement: string): SearchRequest {
  const value = replacement.trim();
  if (!value || value === term) return request;
  return { ...request, [kind]: request[kind].map((current) => current === term ? value : current) };
}
