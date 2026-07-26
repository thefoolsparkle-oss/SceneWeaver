// @vitest-environment jsdom
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, cleanup, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';

import Jobs from './Jobs';

const { cancelJob, listJobs, pauseJob, progressListener, resumeJob, retryJob } = vi.hoisted(() => {
  let listener: ((event: { payload: unknown }) => void) | undefined;
  return {
    cancelJob: vi.fn(),
    listJobs: vi.fn(),
    pauseJob: vi.fn(),
    progressListener: {
      emit(payload: unknown) {
        listener?.({ payload });
      },
      set(next: (event: { payload: unknown }) => void) {
        listener = next;
      },
    },
    resumeJob: vi.fn(),
    retryJob: vi.fn(),
  };
});

vi.mock('@/api', () => ({ cancelJob, listJobs, pauseJob, resumeJob, retryJob }));
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockImplementation(async (_event: string, callback: (event: { payload: unknown }) => void) => {
    progressListener.set(callback);
    return () => undefined;
  }),
}));

const runningJob = {
  id: 'scan-1',
  job_type: 'scan' as const,
  library_id: 'library-1',
  asset_id: null,
  status: 'running' as const,
  priority: 0,
  progress: 0.2,
  current_step: '扫描中',
  checkpoint_json: JSON.stringify({ processed: 2, total: 10, errors: 0 }),
  error_code: null,
  error_message: null,
  started_at: 1,
  finished_at: null,
  created_at: 1,
  updated_at: 1,
};

function renderJobs() {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return render(
    <QueryClientProvider client={client}>
      <Jobs />
    </QueryClientProvider>,
  );
}

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe('Jobs page interactions', () => {
  it('sends pause and resume commands for the selected task', async () => {
    pauseJob.mockResolvedValue({ ...runningJob, status: 'paused' });
    listJobs.mockResolvedValue([runningJob]);
    const user = userEvent.setup();
    renderJobs();

    await user.click(await screen.findByRole('button', { name: '暂停' }));
    await waitFor(() => expect(pauseJob).toHaveBeenCalledWith('scan-1', expect.anything()));

    cleanup();
    resumeJob.mockResolvedValue({ ...runningJob, status: 'pending' });
    listJobs.mockResolvedValue([{ ...runningJob, status: 'paused' as const }]);
    renderJobs();
    await user.click(await screen.findByRole('button', { name: '恢复' }));
    await waitFor(() => expect(resumeJob).toHaveBeenCalledWith('scan-1', expect.anything()));
  });

  it('applies scan progress events to the visible job', async () => {
    listJobs.mockResolvedValue([runningJob]);
    renderJobs();
    await screen.findByText('运行中');

    await waitFor(() => expect(progressListener.emit).toBeTypeOf('function'));
    act(() => {
      progressListener.emit({
        job_id: 'scan-1',
        library_id: 'library-1',
        status: 'paused',
        progress: 0.6,
        current_step: '暂停中',
        processed: 6,
        total: 10,
        errors: 1,
      });
    });

    expect(await screen.findByText('已暂停')).toBeTruthy();
    expect(screen.getByText('暂停中')).toBeTruthy();
    expect(screen.getByText('已处理 6 / 10 · 错误 1')).toBeTruthy();
  });
});
