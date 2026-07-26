import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { remote } from 'webdriverio';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const appBinary = path.join(root, 'src-tauri', 'target', 'debug', 'sceneweaver.exe');
const port = 4445;
const devServerUrl = 'http://127.0.0.1:1420';

function delay(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

async function waitForUrl(url, label) {
  let lastError = 'no response';
  for (let attempt = 0; attempt < 60; attempt += 1) {
    try {
      const response = await fetch(url);
      if (response.ok) return;
      lastError = `HTTP ${response.status}`;
    } catch (error) {
      lastError = error instanceof Error ? error.message : String(error);
    }
    await delay(250);
  }
  throw new Error(`${label} did not start: ${lastError}`);
}

const vite = spawn(process.execPath, [path.join(root, 'node_modules', 'vite', 'bin', 'vite.js'), '--host', '127.0.0.1', '--port', '1420', '--strictPort'], {
  cwd: root,
  stdio: 'pipe',
  windowsHide: true,
});
let viteOutput = '';
vite.stdout.on('data', (chunk) => { viteOutput += String(chunk); });
vite.stderr.on('data', (chunk) => { viteOutput += String(chunk); });

let app;
let appOutput = '';
let browser;
const appDataDir = await mkdtemp(path.join(tmpdir(), 'sceneweaver-e2e-'));
try {
  await waitForUrl(devServerUrl, 'Vite development server');
  app = spawn(appBinary, [], {
    cwd: root,
    env: {
      ...process.env,
      TAURI_DATA_DIR: appDataDir,
      TAURI_WEBDRIVER_PORT: String(port),
    },
    stdio: 'pipe',
    windowsHide: true,
  });
  app.stdout.on('data', (chunk) => { appOutput += String(chunk); });
  app.stderr.on('data', (chunk) => { appOutput += String(chunk); });

  await waitForUrl(`http://127.0.0.1:${port}/status`, 'embedded WebDriver');
  browser = await remote({
    hostname: '127.0.0.1',
    port,
    capabilities: {},
    connectionRetryCount: 0,
    logLevel: 'silent',
  });

  const appName = await browser.$('[data-testid="app-name"]');
  await appName.waitForDisplayed({ timeout: 15_000 });
  assert.equal(await browser.getTitle(), 'SceneWeaver');

  await (await browser.$('[data-testid="nav-search"]')).click();
  const heading = await browser.$('[data-testid="search-heading"]');
  await heading.waitForDisplayed();
  assert.ok((await heading.getText()).trim().length > 0);
  console.log('desktop e2e passed: application launch and search navigation');
} finally {
  await browser?.deleteSession().catch(() => undefined);
  app?.kill();
  vite.kill();
  await Promise.all([
    app ? Promise.race([new Promise((resolve) => app.once('exit', resolve)), delay(5_000)]) : Promise.resolve(),
    Promise.race([new Promise((resolve) => vite.once('exit', resolve)), delay(5_000)]),
  ]);
  if (app?.exitCode === null) app.kill('SIGKILL');
  if (vite.exitCode === null) vite.kill('SIGKILL');
  if (app?.exitCode !== 0 && app?.exitCode !== null) {
    console.error(`SceneWeaver exited with ${app.exitCode}: ${appOutput}`);
  }
  if (vite.exitCode !== 0 && vite.exitCode !== null) {
    console.error(`Vite exited with ${vite.exitCode}: ${viteOutput}`);
  }
  await rm(appDataDir, { recursive: true, force: true });
}
