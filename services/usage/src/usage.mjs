// PecoFence usage statistics, the receiving side. The app posts one report per installation
// and day to `POST /usage`; the Lambda function (index.mjs) checks it here and writes it to
// the bucket as `day=<day>/<id>.json`, one line of JSON, where Athena reads it. A later upload
// of the same day replaces the earlier one.
//
// Nothing about the sender is kept: the IP address is not stored, and neither the body nor the
// id is logged.

const MAX_BODY = 256 * 1024;
const MAX_EVENTS = 2000;
const MAX_STRING = 64;
const NAME = /^[a-z0-9_.]{1,60}$/;
const DAY_MS = 24 * 60 * 60 * 1000;

const isObject = (v) => v !== null && typeof v === 'object' && !Array.isArray(v);

/** Checks a report; returns what is wrong with it, or null when it may be stored. */
export function validate(report, now = Date.now()) {
  if (!isObject(report)) return 'report must be an object';
  const { v, id, day, events, ...rest } = report;
  if (v !== 1) return 'unknown version';
  if (typeof id !== 'string' || !/^[0-9a-f]{32}$/.test(id)) return 'invalid id';

  // The day is the sender's local date: up to 8 days old (the app keeps unsent days for 7),
  // and up to 1 day ahead of UTC.
  if (typeof day !== 'string' || !/^\d{4}-\d{2}-\d{2}$/.test(day)) return 'invalid day';
  const time = Date.parse(`${day}T00:00:00Z`);
  if (Number.isNaN(time) || new Date(time).toISOString().slice(0, 10) !== day) return 'invalid day';
  const age = (Math.floor(now / DAY_MS) * DAY_MS - time) / DAY_MS;
  if (age > 8 || age < -1) return 'day out of range';

  // [second of the day, name] or [second of the day, name, key]
  if (!Array.isArray(events) || events.length > MAX_EVENTS) return 'invalid events';
  for (const event of events) {
    if (!Array.isArray(event) || event.length < 2 || event.length > 3) return 'invalid event';
    const [second, ...names] = event;
    if (!Number.isInteger(second) || second < 0 || second >= 86400) return 'invalid event';
    if (!names.every((name) => typeof name === 'string' && NAME.test(name))) return 'invalid event';
  }

  // Everything else (app, edition, windows, language; settings, state, health).
  for (const [name, value] of Object.entries(rest)) {
    if (!/^[a-z]{1,20}$/.test(name)) return 'invalid field';
    if (!isObject(value) && !(typeof value === 'string' && value.length <= MAX_STRING)) return 'invalid field';
  }
  return null;
}

const reply = (statusCode, error, headers = {}) =>
  error ? { statusCode, headers: { 'content-type': 'application/json', ...headers }, body: JSON.stringify({ error }) } : { statusCode };

/** Answers one request of the function URL. `put(key, body)` writes an object to the bucket. */
export async function receive(request, put, now = Date.now()) {
  if (request.rawPath !== '/usage') return reply(404, 'not found');
  if (request.requestContext.http.method !== 'POST') return reply(405, 'POST only', { allow: 'POST' });
  const body = request.isBase64Encoded ? Buffer.from(request.body, 'base64').toString() : request.body || '';
  if (Buffer.byteLength(body) > MAX_BODY) return reply(413, 'too large');
  let report;
  try {
    report = JSON.parse(body);
  } catch {
    return reply(400, 'invalid JSON');
  }
  const error = validate(report, now);
  if (error) return reply(400, error);
  try {
    await put(`day=${report.day}/${report.id}.json`, JSON.stringify(report) + '\n');
  } catch (e) {
    // The name of the error only: its message can hold the object's key, and with it the id.
    console.error('could not store a report:', e.name);
    return reply(500, 'could not store the report');
  }
  return reply(204);
}
