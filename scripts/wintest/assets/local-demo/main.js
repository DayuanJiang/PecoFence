// The test plugin of tests/plugin-local.ps1: one row per thing it was told or read.
const rows = { hint: 'Drop here' };
const show = () => pf.show({ rows: Object.values(rows).map((text) => ({ text })) });
const token = () => { rows.token = 'token=' + (pf.settings.token === null ? 'none' : pf.settings.token); };
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
  rows.drop = `drop files=${files} text=${d.text === null ? 'none' : d.text} url=${d.url || 'none'}`;
  for (const k of ['text0', 'b64', 'saved']) delete rows[k];
  show();
  if (!d.files.length) return;
  // After the handler too: each call reads the last drop's file.
  pf.after(1, () => {
    tried('text0', () => pf.drop.text(0));
    tried('b64', () => pf.drop.read(0));
    tried('saved', () => pf.drop.save(0, 'saved.bin').then(() => 'ok'));
  });
});
