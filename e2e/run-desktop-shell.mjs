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

async function invoke(command, args = {}) {
  const result = await browser.executeAsync((name, input, done) => {
    const invokeCommand = window.__TAURI_INTERNALS__?.invoke;
    if (!invokeCommand) {
      done({ ok: false, message: 'Tauri IPC bridge is unavailable' });
      return;
    }
    invokeCommand(name, input).then(
      (value) => done({ ok: true, value }),
      (error) => done({ ok: false, message: String(error) }),
    );
  }, command, args);
  if (!result.ok) throw new Error(`${command} failed: ${result.message}`);
  return result.value;
}

try {
  await waitForUrl(devServerUrl, 'Vite development server');
  app = spawn(appBinary, [], {
    cwd: root,
    env: {
      ...process.env,
      SCENEWEAVER_E2E_DATA_DIR: appDataDir,
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

  await (await browser.$('[data-testid="nav-selects"]')).click();
  const selectsHeading = await browser.$('[data-testid="selects-heading"]');
  await selectsHeading.waitForDisplayed();
  const collectionName = 'E2E 自定义选片';
  await (await browser.$('[data-testid="select-collection-name"]')).setValue(collectionName);
  await (await browser.$('[data-testid="create-select-collection"]')).click();
  const collection = await browser.$(`button=${collectionName}`);
  await collection.waitForDisplayed();

  await browser.refresh();
  await (await browser.$(`button=${collectionName}`)).waitForDisplayed();

  const fixture = await invoke('seed_e2e_segment_select_fixture');
  await (await browser.$('[data-testid="nav-libraries"]')).click();
  const fixtureLibrary = await browser.$('a=E2E 片段素材库');
  await fixtureLibrary.waitForDisplayed();
  await fixtureLibrary.click();
  await browser.refresh();
  const viewSegments = await browser.$('[title="查看镜头片段"]');
  await viewSegments.waitForExist();
  await browser.execute((element) => element.click(), viewSegments);
  const targetCollection = await browser.$('[aria-label="加入片段到选片集合"]');
  await targetCollection.waitForDisplayed();
  await browser.execute((element, value) => {
    const valueSetter = Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, 'value').set;
    valueSetter.call(element, value);
    element.dispatchEvent(new Event('change', { bubbles: true }));
  }, targetCollection, fixture.collection_id);
  await (await browser.$('button=加入选片')).click();
  await (await browser.$('button=已加入选片')).waitForDisplayed();
  const fixtureItems = await invoke('list_select_items', { collectionId: fixture.collection_id });
  assert.equal(fixtureItems.length, 1, `fixture segment was not persisted: ${JSON.stringify(fixtureItems)}`);
  assert.equal(fixtureItems[0].segment_id, fixture.segment_id);

  await (await browser.$('[data-testid="nav-selects"]')).click();
  await browser.refresh();
  const fixtureCollection = await browser.$('button=E2E 片段集合');
  await fixtureCollection.waitForDisplayed();
  await fixtureCollection.click();
  await browser.pause(250);
  const selectsText = await (await browser.$('body')).getText();
  assert.ok(selectsText.includes('fixture.mp4'), `custom segment card missing from Selects:\n${selectsText}`);
  console.log('desktop e2e passed: application launch, search navigation, custom Selects persistence, and custom segment selects');
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
