// The test plugin of tests/plugin-local.ps1: one row per thing it was told or read. The first
// row is tall and has an id, for a drop on a row.
const rows = { hint: 'Drop here' };
const show = () => pf.show({
  rows: Object.entries(rows).map(([k, text]) => (k === 'hint' ? { id: k, text, size: 'huge' } : { text })),
});
const token = () => { rows.token = 'token=' + (pf.settings.token === null ? 'none' : pf.settings.token); };
// A long text or base64 by its length.
const short = (s) => (s.length > 64 ? 'length ' + s.length : s);
let events = 0;
const apps = async () => {
  const [running, front] = await Promise.all([pf.apps.running(), pf.apps.front()]);
  rows.apps = `running=${running.join(',') || 'none'} front=${front || 'none'} events=${events}`;
  show();
};
const tried = async (what, f) => {
  try { rows[what] = `${what}: ${await f()}`; } catch (e) { rows[what] = `${what} refused: ${e.message}`; }
  show();
};
pf.on('start', () => { token(); show(); apps(); });
pf.on('settings', () => { token(); show(); });
pf.on('apps', (a) => {
  events++;
  rows.event = `event running=${a.running.join(',') || 'none'} front=${a.front || 'none'}`;
  apps();
});
pf.on('drop', (d) => {
  const files = d.files.map((f) => `${f.name}:${f.size}`).join(',') || 'none';
  rows.drop = `drop files=${files} text=${d.text === null ? 'none' : d.text} url=${d.url || 'none'} row=${d.row}`;
  for (const k of ['text0', 'b64', 'saved']) delete rows[k];
  show();
  if (!d.files.length) return;
  // After the handler too: each call reads the last drop's file.
  pf.after(1, () => {
    tried('text0', () => pf.drop.text(0).then(short));
    tried('b64', () => pf.drop.read(0).then(short));
    tried('saved', () => pf.drop.save(0, 'saved.bin').then(() => 'ok'));
  });
});
