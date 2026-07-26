// @vitest-environment jsdom
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter, Route, Routes } from 'react-router-dom';
import { afterEach, describe, expect, it, vi } from 'vitest';

import LibraryDetail from './LibraryDetail';

const { addSegmentToSelectCollection, getLibrary, listAssets, listSegments, listSelectCollections } = vi.hoisted(() => ({
  addSegmentToSelectCollection: vi.fn(),
  getLibrary: vi.fn(),
  listAssets: vi.fn(),
  listSegments: vi.fn(),
  listSelectCollections: vi.fn(),
}));

vi.mock('@/api', () => ({
  addSegmentToDefaultSelects: vi.fn(),
  addSegmentToSelectCollection,
  copyToClipboard: vi.fn(),
  getLibrary,
  listAssets,
  listSegments,
  listSelectCollections,
  segmentPreviewDataUrl: vi.fn(),
}));

vi.mock('@/components/MediaGrid', () => ({
  MediaGrid: ({ onViewSegments }: { onViewSegments: (assetId: string) => void }) => <button onClick={() => onViewSegments('asset-1')}>打开片段</button>,
}));

function renderPage() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } });
  return render(<QueryClientProvider client={client}><MemoryRouter initialEntries={['/libraries/library-1']}><Routes><Route path="/libraries/:id" element={<LibraryDetail />} /></Routes></MemoryRouter></QueryClientProvider>);
}

afterEach(() => vi.clearAllMocks());

describe('LibraryDetail segment selects', () => {
  it('adds a segment to the selected custom collection', async () => {
    getLibrary.mockResolvedValue({ id: 'library-1', name: '测试素材库', root_path: 'C:\\media', status: 'idle' });
    listAssets.mockResolvedValue([]);
    listSegments.mockResolvedValue([{ id: 'segment-1', asset_id: 'asset-1', segment_type: 'scene', segment_index: 0, start_ms: 1_000, end_ms: 4_000, duration_ms: 3_000, representative_frame_path: null, thumbnail_path: null, preview_path: null, quality_score: 0.9, subtitle_present: false, game_ui: false, black_frame_score: 0, blur_score: 0, embedding_ref: null, created_at: 1, updated_at: 1 }]);
    listSelectCollections.mockResolvedValue([{ id: 'custom-1', name: '剧情候选', description: null, created_at: 1, updated_at: 1 }]);
    addSegmentToSelectCollection.mockResolvedValue(undefined);
    const user = userEvent.setup();
    renderPage();

    await user.click(await screen.findByRole('button', { name: '打开片段' }));
    await user.selectOptions(await screen.findByLabelText('加入片段到选片集合'), 'custom-1');
    await user.click(screen.getByRole('button', { name: '加入选片' }));

    await waitFor(() => expect(addSegmentToSelectCollection).toHaveBeenCalledWith('custom-1', 'asset-1', 'segment-1'));
  });
});
