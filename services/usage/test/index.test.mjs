import assert from 'node:assert/strict';
import { test } from 'node:test';
import { receive, validate } from '../src/index.mjs';

const NOW = Date.parse('2026-11-14T03:00:00Z');

const report = (overrides = {}) => ({
  v: 1,
  id: '5f0c2a9e7b1d4c38a6e0f1b2c3d4e5f6',
  day: '2026-11-14',
  app: '0.3.4',
  edition: 'setup',
  windows: '10.0.26200',
  language: 'zh-CN',
  settings: { theme: 'dark', hideRealIcons: true },
  state: { fences: 6, items: 73 },
  health: { startupMs: 420, memoryMB: 31, crashed: false },
  events: [[32400, 'start', 'at_sign_in'], [32655, 'hotkey', 'peek'], [32671, 'launch_items']],
  ...overrides,
});

/** A function URL request, the way Lambda hands it to the function. */
const request = (body, { method = 'POST', path = '/usage', base64 = false } = {}) => {
  const text = typeof body === 'string' ? body : JSON.stringify(body);
  return {
    rawPath: path,
    headers: { 'content-type': 'application/json', 'x-forwarded-for': '203.0.113.7' },
    requestContext: { http: { method, path, sourceIp: '203.0.113.7' } },
    body: base64 ? Buffer.from(text).toString('base64') : text,
    isBase64Encoded: base64,
  };
};

/** Runs the function with the bucket replaced by a list of the objects it was asked to write. */
async function run(req, put) {
  const objects = [];
  const logged = [];
  const realError = console.error;
  console.error = (...args) => logged.push(args.join(' '));
  try {
    const response = await receive(req, put || (async (key, body) => objects.push({ key, body })), NOW);
    return { response, objects, logged };
  } finally {
    console.error = realError;
  }
}

test('a valid report is stored under its day and id, as one line of JSON', async () => {
  const { response, objects, logged } = await run(request(report()));
  assert.deepEqual(response, { statusCode: 204 });
  assert.equal(objects.length, 1);
  assert.equal(objects[0].key, 'day=2026-11-14/5f0c2a9e7b1d4c38a6e0f1b2c3d4e5f6.json');
  assert.equal(objects[0].body, JSON.stringify(report()) + '\n');
  assert.equal(objects[0].body.trim().split('\n').length, 1);
  // Nothing of the sender is stored or logged.
  assert.doesNotMatch(objects[0].body, /203\.0\.113\.7/);
  assert.deepEqual(logged, []);
});

test('a body sent with line breaks, or base64 encoded by Lambda, is stored the same way', async () => {
  const pretty = await run(request(JSON.stringify(report(), null, 2)));
  const base64 = await run(request(report(), { base64: true }));
  assert.equal(pretty.response.statusCode, 204);
  assert.equal(pretty.objects[0].body, JSON.stringify(report()) + '\n');
  assert.deepEqual(base64.objects, pretty.objects);
});

test('the day may be 8 days old or 1 day ahead of UTC, not more', () => {
  for (const [day, ok] of [
    ['2026-11-06', true],
    ['2026-11-15', true],
    ['2026-11-05', false],
    ['2026-11-16', false],
    ['2026-02-30', false],
    ['2026-13-01', false],
    ['14/11/2026', false],
    [20261114, false],
  ]) {
    assert.equal(validate(report({ day }), NOW) === null, ok, String(day));
  }
});

test('reports that break a rule are refused and nothing is stored', async () => {
  const names = (n) => Array.from({ length: n }, (_, i) => [i, 'launch_items']);
  for (const body of [
    report({ v: 2 }),
    report({ id: '5F0C2A9E7B1D4C38A6E0F1B2C3D4E5F6' }),
    report({ id: '5f0c2a9e7b1d4c38a6e0f1b2c3d4e5f' }),
    report({ id: '../../../../etc/passwd/aaaaaaaaaa' }),
    report({ events: 'start' }),
    report({ events: names(2001) }),
    report({ events: [[32400]] }),
    report({ events: [[32400, 'start', 'by_hand', 'extra']] }),
    report({ events: [['32400', 'start']] }),
    report({ events: [[86400, 'start']] }),
    report({ events: [[1.5, 'start']] }),
    report({ events: [[32400, 'Start']] }),
    report({ events: [[32400, 'open', 'C:\\Users\\me\\taxes.xlsx']] }),
    report({ events: [[32400, 'x'.repeat(61)]] }),
    report({ events: [[32400, 'start', 7]] }),
    report({ app: 'x'.repeat(65) }),
    report({ app: 34 }),
    report({ settings: ['dark'] }),
    report({ state: null }),
    report({ Extra: 'x' }),
    [report()],
    'null',
    '{not json',
    '',
  ]) {
    const { response, objects } = await run(request(body));
    assert.equal(response.statusCode, 400, JSON.stringify(body).slice(0, 80));
    assert.match(JSON.parse(response.body).error, /\w/);
    assert.equal(objects.length, 0);
  }
  assert.equal(validate(report({ events: names(2000) }), NOW), null);
  assert.equal(validate(report({ events: [] }), NOW), null);
});

test('protocol errors', async () => {
  const big = report({ settings: { filler: 'x'.repeat(256 * 1024) } });
  assert.equal((await run(request(big))).response.statusCode, 413);
  const get = await run(request('', { method: 'GET' }));
  assert.equal(get.response.statusCode, 405);
  assert.equal(get.response.headers.allow, 'POST');
  assert.equal((await run(request(report(), { path: '/' }))).response.statusCode, 404);
  assert.equal((await run(request(report(), { path: '/usage/x' }))).response.statusCode, 404);
});

test('a failed write answers 500 and logs neither the id nor the body', async () => {
  const failing = async (key) => {
    const error = new Error(`not authorized to write ${key}`);
    error.name = 'AccessDenied';
    throw error;
  };
  const { response, logged } = await run(request(report()), failing);
  assert.equal(response.statusCode, 500);
  assert.deepEqual(logged, ['could not store a report: AccessDenied']);
});
