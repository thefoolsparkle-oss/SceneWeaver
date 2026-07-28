// @vitest-environment jsdom
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { cleanup, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';

import Entities from './Entities';

const { addEntityReferenceImage, listEntities, listEntityReferences, open } = vi.hoisted(() => ({
  addEntityReferenceImage: vi.fn(),
  listEntities: vi.fn(),
  listEntityReferences: vi.fn(),
  open: vi.fn(),
}));

vi.mock('@/api', () => ({
  addEntityReferenceImage,
  createEntity: vi.fn(),
  findAssetsForEntity: vi.fn(),
  listEntities,
  listEntityReferences,
  removeEntityReference: vi.fn(),
  setEntityAssetFeedback: vi.fn(),
}));
vi.mock('@tauri-apps/plugin-dialog', () => ({ open }));
vi.mock('@/components/MediaGrid', () => ({ MediaGrid: () => null }));

const entity = {
  id: 'entity-e2e', entity_type: 'character', name: 'Entity E2E', description: null,
  aliases: [], pack_id: null, created_at: 1, updated_at: 1,
};

function renderEntities() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } });
  return render(<QueryClientProvider client={client}><Entities /></QueryClientProvider>);
}

afterEach(() => {
  cleanup();
  delete (window as Window & { __SCENEWEAVER_E2E_ENTITY_REFERENCE_PATH__?: string }).__SCENEWEAVER_E2E_ENTITY_REFERENCE_PATH__;
  addEntityReferenceImage.mockReset();
  addEntityReferenceImage.mockResolvedValue({});
  listEntities.mockReset();
  listEntities.mockResolvedValue([entity]);
  listEntityReferences.mockReset();
  listEntityReferences.mockResolvedValue([]);
  open.mockReset();
});

describe('Entity reference interactions', () => {
  it('uses the development-only reference path for positive and negative references without opening the system picker', async () => {
    listEntities.mockResolvedValue([entity]);
    listEntityReferences.mockResolvedValue([]);
    addEntityReferenceImage.mockResolvedValue({});
    const user = userEvent.setup();
    renderEntities();

    await screen.findByText(entity.name);
    (window as Window & { __SCENEWEAVER_E2E_ENTITY_REFERENCE_PATH__?: string }).__SCENEWEAVER_E2E_ENTITY_REFERENCE_PATH__ = 'C:\\fixtures\\positive.png';
    await user.click(screen.getByTestId(`entity-positive-reference-${entity.id}`));
    await waitFor(() => expect(addEntityReferenceImage).toHaveBeenCalled());
    expect(addEntityReferenceImage.mock.calls[0]?.slice(0, 3)).toEqual([entity.id, 'C:\\fixtures\\positive.png', true]);

    (window as Window & { __SCENEWEAVER_E2E_ENTITY_REFERENCE_PATH__?: string }).__SCENEWEAVER_E2E_ENTITY_REFERENCE_PATH__ = 'C:\\fixtures\\negative.png';
    await user.click(screen.getByTestId(`entity-negative-reference-${entity.id}`));
    await waitFor(() => expect(addEntityReferenceImage).toHaveBeenCalledTimes(2));
    expect(addEntityReferenceImage.mock.calls[1]?.slice(0, 3)).toEqual([entity.id, 'C:\\fixtures\\negative.png', false]);
    expect(open).not.toHaveBeenCalled();
  });
});
