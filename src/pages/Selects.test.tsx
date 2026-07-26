// @vitest-environment jsdom
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { cleanup, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';

import Selects from './Selects';

const { exportSelectCollectionCsv, listSelectCollections, listSelectItems, reorderSelectItem, save, updateSelectItem } = vi.hoisted(() => ({
  exportSelectCollectionCsv: vi.fn(),
  listSelectCollections: vi.fn(),
  listSelectItems: vi.fn(),
  reorderSelectItem: vi.fn(),
  save: vi.fn(),
  updateSelectItem: vi.fn(),
}));

vi.mock('@/api', () => ({
  addSelectItemTag: vi.fn(), addSelectItemTagBatch: vi.fn(), createSelectCollection: vi.fn(),
  exportDefaultSelectsCsv: vi.fn(), exportDefaultSelectsEdl: vi.fn(), exportDefaultSelectsFcpxml: vi.fn(), exportDefaultSelectsJson: vi.fn(),
  exportSelectCollectionContactSheet: vi.fn(), exportSelectCollectionContactSheetHtml: vi.fn(), exportSelectCollectionCsv,
  exportSelectCollectionEdl: vi.fn(), exportSelectCollectionFcpxml: vi.fn(), exportSelectCollectionJson: vi.fn(),
  listSelectCollections, listSelectItems, moveSelectItem: vi.fn(), openAsset: vi.fn(), removeSelectItem: vi.fn(), removeSelectItemTag: vi.fn(),
  reorderSelectItem, updateSelectItem,
}));
vi.mock('@tauri-apps/plugin-dialog', () => ({ save }));

const collection = { id: 'custom-1', name: '剧情候选', description: '自定义集合', created_at: 1, updated_at: 1 };
const item = (id: string, position: number) => ({
  id, collection_id: collection.id, asset_id: `asset-${id}`, segment_id: `segment-${id}`, position,
  rating: null, note: null, recommended_in_ms: null, recommended_out_ms: null, tags: [], created_at: 1, updated_at: 1,
  asset: { id: `asset-${id}`, library_id: 'library-1', media_type: 'video', file_path: `C:\\media\\${id}.mp4`, normalized_path: `C:\\media\\${id}.mp4`, file_name: `${id}.mp4`, extension: 'mp4', size_bytes: 1, modified_at: 1, quick_fingerprint: 'fingerprint', full_hash: null, duration_ms: 10_000, width: 1920, height: 1080, fps: 24, codec: 'h264', capture_time: null, status: 'ready', index_level: 1, analysis_version: 1, created_at: 1, updated_at: 1 },
  segment: { id: `segment-${id}`, asset_id: `asset-${id}`, segment_type: 'scene', segment_index: position, start_ms: 1_000, end_ms: 4_000, duration_ms: 3_000, quality_score: 0.9, subtitle_present: false, game_ui: false },
});

function renderSelects() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } });
  return render(<QueryClientProvider client={client}><Selects /></QueryClientProvider>);
}

afterEach(() => {
  delete (window as Window & { __SCENEWEAVER_E2E_EXPORT_PATH__?: string }).__SCENEWEAVER_E2E_EXPORT_PATH__;
  cleanup(); vi.clearAllMocks();
});

describe('Selects page interactions', () => {
  it('exports a custom collection and persists its explicit order', async () => {
    listSelectCollections.mockResolvedValue([collection]);
    listSelectItems.mockResolvedValue([item('first', 0), item('second', 1)]);
    save.mockResolvedValue('C:\\exports\\story.csv');
    exportSelectCollectionCsv.mockResolvedValue(undefined);
    reorderSelectItem.mockResolvedValue(undefined);
    const user = userEvent.setup();
    renderSelects();

    await user.click(await screen.findByRole('button', { name: '导出 CSV' }));
    await waitFor(() => expect(exportSelectCollectionCsv).toHaveBeenCalledWith('custom-1', 'C:\\exports\\story.csv'));

    await user.click(screen.getAllByRole('button', { name: '下移' })[0]);
    await waitFor(() => expect(reorderSelectItem).toHaveBeenCalledWith('first', 1));
  });

  it('uses the development-only E2E export path when its extension matches', async () => {
    listSelectCollections.mockResolvedValue([collection]);
    listSelectItems.mockResolvedValue([item('first', 0)]);
    exportSelectCollectionCsv.mockResolvedValue(undefined);
    (window as Window & { __SCENEWEAVER_E2E_EXPORT_PATH__?: string }).__SCENEWEAVER_E2E_EXPORT_PATH__ = 'C:\\exports\\desktop-e2e.csv';
    const user = userEvent.setup();
    renderSelects();

    await user.click(await screen.findByTestId('export-csv'));
    await waitFor(() => expect(exportSelectCollectionCsv).toHaveBeenCalledWith('custom-1', 'C:\\exports\\desktop-e2e.csv'));
    expect(save).not.toHaveBeenCalled();
  });

  it('converts recommended in/out points from seconds to milliseconds', async () => {
    listSelectCollections.mockResolvedValue([collection]);
    listSelectItems.mockResolvedValue([item('first', 0)]);
    updateSelectItem.mockResolvedValue(item('first', 0));
    const user = userEvent.setup();
    renderSelects();

    const inPoint = await screen.findByRole('textbox', { name: '推荐入点（秒）' });
    const outPoint = screen.getByRole('textbox', { name: '推荐出点（秒）' });
    await user.type(inPoint, '1.25');
    await user.type(outPoint, '3.5');
    await user.click(screen.getByRole('button', { name: '保存标注' }));

    await waitFor(() => expect(updateSelectItem).toHaveBeenCalledWith('first', expect.objectContaining({ recommended_in_ms: 1_250, recommended_out_ms: 3_500 })));
  });
});
