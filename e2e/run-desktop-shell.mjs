import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { deflateSync } from 'node:zlib';
import { remote } from 'webdriverio';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const appBinary = path.join(root, 'src-tauri', 'target', 'debug', 'sceneweaver.exe');
const port = 4445;
const devServerUrl = 'http://127.0.0.1:1420';

function crc32(bytes) {
  let crc = 0xffff_ffff;
  for (const byte of bytes) {
    crc ^= byte;
    for (let bit = 0; bit < 8; bit += 1) {
      crc = (crc >>> 1) ^ (crc & 1 ? 0xedb8_8320 : 0);
    }
  }
  return (crc ^ 0xffff_ffff) >>> 0;
}

function pngChunk(type, data) {
  const typeBytes = Buffer.from(type, 'ascii');
  const length = Buffer.alloc(4);
  length.writeUInt32BE(data.length);
  const checksum = Buffer.alloc(4);
  checksum.writeUInt32BE(crc32(Buffer.concat([typeBytes, data])));
  return Buffer.concat([length, typeBytes, data, checksum]);
}

function createPngFixture(red = 18, green = 52, blue = 86) {
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(1, 0);
  ihdr.writeUInt32BE(1, 4);
  ihdr[8] = 8;
  ihdr[9] = 2;
  const rawPixels = Buffer.from([0, red, green, blue]);
  return Buffer.concat([
    Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
    pngChunk('IHDR', ihdr),
    pngChunk('IDAT', deflateSync(rawPixels)),
    pngChunk('IEND', Buffer.alloc(0)),
  ]);
}

const pngFixture = createPngFixture();
const referencePngFixture = createPngFixture(194, 47, 29);

function delay(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

function runProcess(command, args, label) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, { stdio: 'ignore', windowsHide: true });
    child.once('error', (error) => reject(new Error(`${label} did not start: ${error.message}`)));
    child.once('exit', (code) => {
      if (code === 0) resolve();
      else reject(new Error(`${label} exited with ${code}`));
    });
  });
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

async function readTextFileWhenReady(filePath, label) {
  let lastError = 'file was not created';
  for (let attempt = 0; attempt < 60; attempt += 1) {
    try {
      return await readFile(filePath, 'utf8');
    } catch (error) {
      lastError = error instanceof Error ? error.message : String(error);
    }
    await delay(100);
  }
  throw new Error(`${label} was not written: ${lastError}`);
}

async function waitForBodyText(expected, label) {
  let lastText = '';
  for (let attempt = 0; attempt < 120; attempt += 1) {
    lastText = await (await browser.$('body')).getText();
    if (lastText.includes(expected)) return lastText;
    await delay(100);
  }
  throw new Error(`${label} did not render ${expected}:\n${lastText}`);
}

async function waitForCompletedScan(libraryId, maxAttempts = 120) {
  let lastStatus = 'no scan job found';
  for (let attempt = 0; attempt < maxAttempts; attempt += 1) {
    const jobs = await invoke('list_jobs');
    const job = jobs.find((candidate) => candidate.library_id === libraryId && candidate.job_type === 'scan');
    if (job) {
      lastStatus = job.status;
      if (job.status === 'completed') return job;
      if (job.status === 'failed' || job.status === 'cancelled') {
        throw new Error(`fixture scan ${job.status}: ${job.error_message ?? 'no error message'}`);
      }
    }
    await delay(100);
  }
  throw new Error(`fixture scan did not complete: ${lastStatus}`);
}

async function waitForScanStatus(libraryId, expectedStatuses, maxAttempts = 600) {
  let lastStatus = 'no scan job found';
  for (let attempt = 0; attempt < maxAttempts; attempt += 1) {
    const jobs = await invoke('list_jobs');
    const job = jobs.find((candidate) => candidate.library_id === libraryId && candidate.job_type === 'scan');
    if (job) {
      lastStatus = job.status;
      if (expectedStatuses.includes(job.status)) return job;
      if (job.status === 'failed' || job.status === 'cancelled' || job.status === 'completed') {
        throw new Error(`fixture scan reached ${job.status} before ${expectedStatuses.join(' or ')}`);
      }
    }
    await delay(100);
  }
  throw new Error(`fixture scan did not reach ${expectedStatuses.join(' or ')}: ${lastStatus}`);
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
const mediaFixtureDir = path.join(appDataDir, 'media-fixture');
const mediaFixturePath = path.join(mediaFixtureDir, 'scan-fixture.png');
const referenceFixturePath = path.join(mediaFixtureDir, 'reference-fixture.png');
const resilienceFixtureDir = path.join(appDataDir, '中文 空格 素材库');
const resilienceLongFixturePath = path.join(
  resilienceFixtureDir,
  ...Array.from(
    { length: 8 },
    (_, index) => `超长路径-${String(index).padStart(2, '0')}-abcdefghijklmnopqrstuvwxyz`,
  ),
  '长路径素材.png',
);
const resilienceCorruptFixturePath = path.join(resilienceFixtureDir, '损坏素材.png');
const pauseFixtureDir = path.join(appDataDir, 'pause-fixture');
const recoveryFixtureDir = path.join(appDataDir, 'recovery-fixture');
const ffmpegPath = process.env.SCENEWEAVER_E2E_FFMPEG_BIN;
const videoFixtureDir = path.join(appDataDir, 'video-fixture');
const videoFixturePath = path.join(videoFixtureDir, 'e2e-video.mp4');
await mkdir(mediaFixtureDir, { recursive: true });
await writeFile(mediaFixturePath, pngFixture);
await writeFile(referenceFixturePath, referencePngFixture);
await mkdir(path.dirname(resilienceLongFixturePath), { recursive: true });
await writeFile(resilienceLongFixturePath, pngFixture);
await writeFile(resilienceCorruptFixturePath, 'not an image');
assert.ok(resilienceLongFixturePath.length > 260, `fixture path is not long enough: ${resilienceLongFixturePath.length}`);
await mkdir(pauseFixtureDir, { recursive: true });
await Promise.all(Array.from({ length: 200 }, (_, index) => writeFile(
  path.join(pauseFixtureDir, `pause-fixture-${String(index).padStart(3, '0')}.png`),
  pngFixture,
)));
await mkdir(recoveryFixtureDir, { recursive: true });
await Promise.all(Array.from({ length: 1_000 }, (_, index) => writeFile(
  path.join(recoveryFixtureDir, `recovery-fixture-${String(index).padStart(4, '0')}.png`),
  pngFixture,
)));
if (ffmpegPath) {
  await mkdir(videoFixtureDir, { recursive: true });
  await runProcess(ffmpegPath, [
    '-f', 'lavfi', '-i', 'color=c=red:s=64x48:d=1',
    '-f', 'lavfi', '-i', 'color=c=blue:s=64x48:d=1',
    '-filter_complex', '[0:v][1:v]concat=n=2:v=1:a=0',
    '-c:v', 'libx264', '-pix_fmt', 'yuv420p', '-y', videoFixturePath,
  ], 'E2E FFmpeg video fixture');
}

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

function launchApp() {
  app = spawn(appBinary, [], {
    cwd: root,
    env: {
      ...process.env,
      ...(ffmpegPath ? { Path: `${path.dirname(ffmpegPath)};${process.env.Path ?? ''}` } : {}),
      SCENEWEAVER_E2E_DATA_DIR: appDataDir,
      TAURI_WEBDRIVER_PORT: String(port),
    },
    stdio: 'pipe',
    windowsHide: true,
  });
  app.stdout.on('data', (chunk) => { appOutput += String(chunk); });
  app.stderr.on('data', (chunk) => { appOutput += String(chunk); });
}

async function connectToApp() {
  await waitForUrl(`http://127.0.0.1:${port}/status`, 'embedded WebDriver');
  browser = await remote({
    hostname: '127.0.0.1',
    port,
    capabilities: {},
    connectionRetryCount: 0,
    logLevel: 'silent',
  });
}

async function restartAppAfterCrash() {
  await browser?.deleteSession().catch(() => undefined);
  browser = undefined;
  if (app?.exitCode === null && app?.signalCode === null) {
    app.kill('SIGKILL');
    await Promise.race([new Promise((resolve) => app.once('exit', resolve)), delay(5_000)]);
  }
  if (app?.exitCode === null && app?.signalCode === null) {
    throw new Error('SceneWeaver did not stop after simulated crash');
  }
  launchApp();
  await connectToApp();
}

try {
  await waitForUrl(devServerUrl, 'Vite development server');
  launchApp();
  await connectToApp();

  const appName = await browser.$('[data-testid="app-name"]');
  await appName.waitForDisplayed({ timeout: 15_000 });
  assert.equal(await browser.getTitle(), 'SceneWeaver');

  const scannedLibrary = await invoke('create_library', {
    req: { name: 'E2E 扫描素材库', root_path: mediaFixtureDir, index_profile: 'quick' },
  });
  await (await browser.$('[data-testid="nav-libraries"]')).click();
  const scanButton = await browser.$(`[data-testid="scan-library-${scannedLibrary.id}"]`);
  await scanButton.waitForDisplayed();
  await scanButton.click();
  await waitForCompletedScan(scannedLibrary.id);
  await (await browser.$(`[data-testid="open-library-${scannedLibrary.id}"]`)).click();
  await waitForBodyText('scan-fixture.png', 'scanned fixture asset');

  const resilienceLibrary = await invoke('create_library', {
    req: { name: 'E2E 中文长路径素材库', root_path: resilienceFixtureDir, index_profile: 'quick' },
  });
  await (await browser.$('[data-testid="nav-libraries"]')).click();
  await browser.refresh();
  const resilienceScanButton = await browser.$(`[data-testid="scan-library-${resilienceLibrary.id}"]`);
  await resilienceScanButton.waitForDisplayed();
  await resilienceScanButton.click();
  await waitForCompletedScan(resilienceLibrary.id);
  const resilienceAssets = await invoke('list_assets', { libraryId: resilienceLibrary.id });
  assert.equal(resilienceAssets.length, 2, `resilience scan indexed ${resilienceAssets.length} assets instead of two`);
  const longPathAsset = resilienceAssets.find((asset) => asset.file_name === '长路径素材.png');
  assert.ok(longPathAsset, `long-path asset missing: ${JSON.stringify(resilienceAssets)}`);
  assert.ok(longPathAsset.normalized_path.length > 260, `indexed path is not long enough: ${longPathAsset.normalized_path.length}`);
  assert.ok(longPathAsset.normalized_path.includes('中文 空格 素材库'));
  assert.ok(
    longPathAsset.thumbnail_data_url?.startsWith('data:image/'),
    `long-path image thumbnail was not generated: ${JSON.stringify(longPathAsset)}`,
  );
  const corruptAsset = resilienceAssets.find((asset) => asset.file_name === '损坏素材.png');
  assert.ok(corruptAsset, `corrupt asset prevented indexing: ${JSON.stringify(resilienceAssets)}`);
  assert.equal(corruptAsset.thumbnail_data_url, undefined, 'corrupt image unexpectedly produced a thumbnail');
  await (await browser.$(`[data-testid="open-library-${resilienceLibrary.id}"]`)).click();
  await waitForBodyText('长路径素材.png', 'long-path asset');

  const pausableLibrary = await invoke('create_library', {
    req: { name: 'E2E 暂停扫描素材库', root_path: pauseFixtureDir, index_profile: 'quick' },
  });
  await (await browser.$('[data-testid="nav-libraries"]')).click();
  await browser.refresh();
  const pauseScanButton = await browser.$(`[data-testid="scan-library-${pausableLibrary.id}"]`);
  await pauseScanButton.waitForDisplayed();
  await pauseScanButton.click();
  const runningScan = await waitForScanStatus(pausableLibrary.id, ['running']);
  await (await browser.$('[data-testid="nav-jobs"]')).click();
  const pauseJobButton = await browser.$(`[data-testid="pause-job-${runningScan.id}"]`);
  await pauseJobButton.waitForDisplayed({ timeout: 15_000 });
  await browser.execute((element) => element.click(), pauseJobButton);
  await pauseJobButton.waitForExist({ reverse: true, timeout: 15_000 });
  await waitForScanStatus(pausableLibrary.id, ['paused']);
  const resumeJobButton = await browser.$(`[data-testid="resume-job-${runningScan.id}"]`);
  await resumeJobButton.waitForDisplayed({ timeout: 15_000 });
  await browser.execute((element) => element.click(), resumeJobButton);
  await resumeJobButton.waitForExist({ reverse: true, timeout: 15_000 });
  await waitForCompletedScan(pausableLibrary.id);
  const pausedAssets = await invoke('list_assets', { libraryId: pausableLibrary.id });
  assert.equal(pausedAssets.length, 200, `pause/resume scan indexed ${pausedAssets.length} assets instead of 200`);

  const recoveryLibrary = await invoke('create_library', {
    req: { name: 'E2E 崩溃恢复素材库', root_path: recoveryFixtureDir, index_profile: 'quick' },
  });
  await (await browser.$('[data-testid="nav-libraries"]')).click();
  await browser.refresh();
  const recoveryScanButton = await browser.$(`[data-testid="scan-library-${recoveryLibrary.id}"]`);
  await recoveryScanButton.waitForDisplayed();
  await recoveryScanButton.click();
  await waitForScanStatus(recoveryLibrary.id, ['running']);
  await restartAppAfterCrash();
  await waitForCompletedScan(recoveryLibrary.id, 1_200);
  const recoveredAssets = await invoke('list_assets', { libraryId: recoveryLibrary.id });
  assert.equal(recoveredAssets.length, 1_000, `recovered scan indexed ${recoveredAssets.length} assets instead of 1,000`);

  if (ffmpegPath) {
    const videoLibrary = await invoke('create_library', {
      req: { name: 'E2E 视频扫描素材库', root_path: videoFixtureDir, index_profile: 'quick' },
    });
    await (await browser.$('[data-testid="nav-libraries"]')).click();
    await browser.refresh();
    const videoScanButton = await browser.$(`[data-testid="scan-library-${videoLibrary.id}"]`);
    await videoScanButton.waitForDisplayed();
    await videoScanButton.click();
    await waitForCompletedScan(videoLibrary.id);
    const videoAssets = await invoke('list_assets', { libraryId: videoLibrary.id });
    assert.equal(videoAssets.length, 1, `video scan indexed ${videoAssets.length} assets instead of one`);
    assert.equal(videoAssets[0].media_type, 'video');
    assert.ok(videoAssets[0].duration_ms > 0, `video duration missing: ${JSON.stringify(videoAssets[0])}`);
    const videoSegments = await invoke('list_segments', { assetId: videoAssets[0].id });
    assert.ok(videoSegments.length > 0, 'video scan did not create any scene segments');
    const derivedSegment = videoSegments.find((segment) => segment.representative_frame_path && segment.preview_path);
    assert.ok(derivedSegment, `video scan did not persist keyframe and preview paths: ${JSON.stringify(videoSegments)}`);
    assert.ok(Number.isFinite(derivedSegment.quality_score), `video segment quality score missing: ${JSON.stringify(derivedSegment)}`);
    assert.ok(Number.isFinite(derivedSegment.black_frame_score), `video segment black-frame score missing: ${JSON.stringify(derivedSegment)}`);
    assert.ok(Number.isFinite(derivedSegment.blur_score), `video segment blur score missing: ${JSON.stringify(derivedSegment)}`);
    const [keyframe, preview] = await Promise.all([
      readFile(derivedSegment.representative_frame_path),
      readFile(derivedSegment.preview_path),
    ]);
    assert.ok(keyframe.length > 0, 'video keyframe was empty');
    assert.ok(preview.length > 0, 'video preview was empty');
  } else {
    console.log('desktop e2e video scan skipped: set SCENEWEAVER_E2E_FFMPEG_BIN to a local ffmpeg executable to enable it');
  }

  await (await browser.$('[data-testid="nav-search"]')).click();
  const heading = await browser.$('[data-testid="search-heading"]');
  await heading.waitForDisplayed();
  assert.ok((await heading.getText()).trim().length > 0);
  const searchInput = await browser.$('input[placeholder="例如：雨夜、角色、采访"]');
  await searchInput.setValue('scan-fixture');
  await (await browser.$('button=搜索')).click();
  const searchText = await waitForBodyText('关键词结果：1 项', 'keyword search result');
  assert.ok(searchText.includes('scan-fixture.png'), `keyword search omitted indexed fixture:\n${searchText}`);
  assert.ok(searchText.includes('满足必须条件：scan-fixture'), `keyword search omitted match reason:\n${searchText}`);

  await browser.execute((value) => { window.__SCENEWEAVER_E2E_REFERENCE_IMAGE_PATH__ = value; }, referenceFixturePath);
  await (await browser.$('button=参考图')).click();
  const referenceSearchText = await waitForBodyText('参考图相似结果：', 'reference image search result');
  assert.ok(referenceSearchText.includes('reference-fixture.png'), `reference image search omitted matching fixture:\n${referenceSearchText}`);
  await browser.execute(() => { delete window.__SCENEWEAVER_E2E_REFERENCE_IMAGE_PATH__; });

  const entityName = 'E2E 测试实体';
  await (await browser.$('[data-testid="nav-entities"]')).click();
  const entityNameInput = await browser.$('input[placeholder="实体名称"]');
  await entityNameInput.setValue(entityName);
  await (await browser.$('input[placeholder="别名，用逗号分隔"]')).setValue('scan-fixture');
  await (await browser.$('button=创建')).click();
  await waitForBodyText(entityName, 'created entity');
  const entity = (await invoke('list_entities')).find((candidate) => candidate.name === entityName);
  assert.ok(entity, `created entity was not persisted: ${JSON.stringify(await invoke('list_entities'))}`);
  assert.deepEqual(entity.aliases, ['scan-fixture']);

  const visualEntityName = 'E2E 颜色参考实体';
  await (await browser.$('input[placeholder="实体名称"]')).setValue(visualEntityName);
  await (await browser.$('button=创建')).click();
  await waitForBodyText(visualEntityName, 'created visual reference entity');
  const visualEntity = (await invoke('list_entities')).find((candidate) => candidate.name === visualEntityName);
  assert.ok(visualEntity, `visual reference entity was not persisted: ${JSON.stringify(await invoke('list_entities'))}`);
  await browser.execute((value) => { window.__SCENEWEAVER_E2E_ENTITY_REFERENCE_PATH__ = value; }, referenceFixturePath);
  const positiveReferenceButton = await browser.$(`[data-testid="entity-positive-reference-${visualEntity.id}"]`);
  await positiveReferenceButton.click();
  await waitForBodyText('正参考 1 张 · 负参考 0 张', 'positive entity reference count');
  await browser.execute((value) => { window.__SCENEWEAVER_E2E_ENTITY_REFERENCE_PATH__ = value; }, mediaFixturePath);
  const negativeReferenceButton = await browser.$(`[data-testid="entity-negative-reference-${visualEntity.id}"]`);
  await negativeReferenceButton.click();
  await waitForBodyText('正参考 1 张 · 负参考 1 张', 'negative entity reference count');
  const visualReferences = await invoke('list_entity_references', { entityId: visualEntity.id });
  assert.equal(visualReferences.length, 2, `entity references were not persisted: ${JSON.stringify(visualReferences)}`);
  assert.ok(visualReferences.some((reference) => reference.image_path === referenceFixturePath && reference.is_positive));
  assert.ok(visualReferences.some((reference) => reference.image_path === mediaFixturePath && !reference.is_positive));
  await (await browser.$(`[data-testid="entity-find-assets-${visualEntity.id}"]`)).click();
  await waitForBodyText('实体匹配素材', 'entity reference visual results');
  const visualEntityCard = await browser.$(`[data-testid="entity-card-${visualEntity.id}"]`);
  assert.ok((await visualEntityCard.getText()).includes('实体匹配素材'), 'entity visual results did not render in its card');
  const visualMatches = await invoke('find_assets_for_entity', { entityId: visualEntity.id });
  assert.equal(visualMatches[0]?.file_name, 'reference-fixture.png', `positive reference did not rank matching asset first: ${JSON.stringify(visualMatches)}`);
  assert.ok(visualMatches.findIndex((asset) => asset.file_name === 'scan-fixture.png') > 0, `negative reference was not ranked below the positive match: ${JSON.stringify(visualMatches)}`);
  await browser.execute(() => { delete window.__SCENEWEAVER_E2E_ENTITY_REFERENCE_PATH__; });

  await (await browser.$('[data-testid="nav-search"]')).click();
  const entitySelect = await browser.$('#entity-search');
  await browser.execute((element, value) => {
    const valueSetter = Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, 'value').set;
    valueSetter.call(element, value);
    element.dispatchEvent(new Event('change', { bubbles: true }));
  }, entitySelect, entity.id);
  await (await browser.$('button=查找实体素材')).click();
  const entitySearchText = await waitForBodyText('实体匹配结果：1 项', 'entity alias search result');
  assert.ok(entitySearchText.includes('scan-fixture.png'), `entity alias search omitted indexed fixture:\n${entitySearchText}`);
  assert.ok(entitySearchText.includes(`本地实体匹配：${entityName}（名称、别名或正参考图）`), `entity alias search omitted explanation:\n${entitySearchText}`);

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
  const rangeInputs = await browser.$$('[inputmode="decimal"]');
  assert.equal(rangeInputs.length, 2, `custom segment did not render recommended in/out inputs: ${rangeInputs.length}`);
  await rangeInputs[0].setValue('1.5');
  await rangeInputs[1].setValue('2.5');
  await (await browser.$('button=保存标注')).click();
  const updatedFixtureItems = await invoke('list_select_items', { collectionId: fixture.collection_id });
  assert.equal(updatedFixtureItems[0]?.recommended_in_ms, 1_500, `custom segment in point was not persisted: ${JSON.stringify(updatedFixtureItems)}`);
  assert.equal(updatedFixtureItems[0]?.recommended_out_ms, 2_500, `custom segment out point was not persisted: ${JSON.stringify(updatedFixtureItems)}`);
  const exportPath = path.join(appDataDir, 'e2e-custom-segment.csv');
  await browser.execute((value) => { window.__SCENEWEAVER_E2E_EXPORT_PATH__ = value; }, exportPath);
  await (await browser.$('[data-testid="export-csv"]')).click();
  const csv = await readTextFileWhenReady(exportPath, 'custom segment CSV export');
  assert.ok(csv.includes('fixture.mp4'), `CSV export omitted fixture media:\n${csv}`);
  assert.ok(csv.includes('00:00:01.500'), `CSV export omitted the recommended in point:\n${csv}`);
  assert.ok(csv.includes('00:00:02.500'), `CSV export omitted the recommended out point:\n${csv}`);
  const jsonExportPath = path.join(appDataDir, 'e2e-custom-segment.json');
  await browser.execute((value) => { window.__SCENEWEAVER_E2E_EXPORT_PATH__ = value; }, jsonExportPath);
  await (await browser.$('[data-testid="export-json"]')).click();
  const json = await readTextFileWhenReady(jsonExportPath, 'custom segment JSON export');
  assert.ok(json.includes('"start_ms": 1500'), `JSON export omitted the effective in point:\n${json}`);
  assert.ok(json.includes('"end_ms": 2500'), `JSON export omitted the effective out point:\n${json}`);
  const edlExportPath = path.join(appDataDir, 'e2e-custom-segment.edl');
  await browser.execute((value) => { window.__SCENEWEAVER_E2E_EXPORT_PATH__ = value; }, edlExportPath);
  await (await browser.$('[data-testid="export-edl"]')).click();
  const edl = await readTextFileWhenReady(edlExportPath, 'custom segment EDL export');
  assert.ok(edl.includes('00:00:01:15 00:00:02:15'), `EDL export omitted the recommended range:\n${edl}`);
  assert.ok(edl.includes('* FROM CLIP NAME: fixture.mp4'), `EDL export omitted fixture media:\n${edl}`);
  const fcpxmlExportPath = path.join(appDataDir, 'e2e-custom-segment.fcpxml');
  await browser.execute((value) => { window.__SCENEWEAVER_E2E_EXPORT_PATH__ = value; }, fcpxmlExportPath);
  await (await browser.$('[data-testid="export-fcpxml"]')).click();
  const fcpxml = await readTextFileWhenReady(fcpxmlExportPath, 'custom segment FCPXML export');
  assert.ok(fcpxml.includes('<fcpxml version="1.10">'), `FCPXML export has no FCPXML root:\n${fcpxml}`);
  assert.ok(fcpxml.includes('start="1500/1000s" duration="1000/1000s"'), `FCPXML export omitted the recommended range:\n${fcpxml}`);
  const scannedAssets = await invoke('list_assets', { libraryId: scannedLibrary.id });
  const sortableAsset = scannedAssets.find((asset) => asset.file_name === 'scan-fixture.png');
  assert.ok(sortableAsset, `sortable fixture asset missing: ${JSON.stringify(scannedAssets)}`);
  await invoke('add_asset_to_select_collection', { collectionId: fixture.collection_id, assetId: sortableAsset.id });
  await browser.refresh();
  const sortableCollection = await browser.$('button=E2E 片段集合');
  await sortableCollection.waitForDisplayed();
  await sortableCollection.click();
  const beforeReorder = await invoke('list_select_items', { collectionId: fixture.collection_id });
  assert.equal(beforeReorder.length, 2, `custom collection did not contain two sortable items: ${JSON.stringify(beforeReorder)}`);
  const segmentItem = beforeReorder.find((item) => item.segment_id === fixture.segment_id);
  assert.ok(segmentItem, `custom segment item missing before reorder: ${JSON.stringify(beforeReorder)}`);
  await (await browser.$(`[data-testid="select-item-down-${segmentItem.id}"]`)).click();
  let afterReorder = beforeReorder;
  for (let attempt = 0; attempt < 50; attempt += 1) {
    afterReorder = await invoke('list_select_items', { collectionId: fixture.collection_id });
    if (afterReorder[1]?.id === segmentItem.id) break;
    await delay(100);
  }
  assert.equal(afterReorder[1]?.id, segmentItem.id, `real Selects down action did not persist order: ${JSON.stringify(afterReorder)}`);
  const orderedJsonPath = path.join(appDataDir, 'e2e-custom-segment-ordered.json');
  await browser.execute((value) => { window.__SCENEWEAVER_E2E_EXPORT_PATH__ = value; }, orderedJsonPath);
  await (await browser.$('[data-testid="export-json"]')).click();
  const orderedJson = await readTextFileWhenReady(orderedJsonPath, 'ordered custom segment JSON export');
  assert.ok(orderedJson.indexOf('scan-fixture.png') < orderedJson.indexOf('fixture.mp4'), `JSON export did not preserve UI order:\n${orderedJson}`);
  await browser.execute(() => { delete window.__SCENEWEAVER_E2E_EXPORT_PATH__; });
  console.log(`desktop e2e passed: application launch, real PNG library scan, Chinese/spaced long-path and corrupt-media resilience, pause/resume scan workflow, crash recovery scan workflow${ffmpegPath ? ', real video scan' : ''}, no-API-key keyword, reference-image, and entity alias/positive-negative reference search, custom Selects persistence, custom segment selects, sorting, and CSV/JSON/EDL/FCPXML export`);
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
  if (app && app.exitCode !== 0 && app.exitCode !== null) {
    console.error(`SceneWeaver exited with ${app.exitCode}: ${appOutput}`);
  }
  if (vite && vite.exitCode !== 0 && vite.exitCode !== null) {
    console.error(`Vite exited with ${vite.exitCode}: ${viteOutput}`);
  }
  await rm(appDataDir, { recursive: true, force: true });
}
