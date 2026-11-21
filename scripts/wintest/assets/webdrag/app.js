// The test web plugin of tests/plugin-drag.ps1. What PecoFence hands the page of a drop (the
// `drop` event: the zone under its point, the first file's text) and what the browser's own drop
// events saw (never anything from outside: the fence takes every drop, only a card dragged inside the
// page lands there); the drags out of the page end in `dragend` with the effect the target took.
const out = document.getElementById('out');
const state = { drop: 'none', text0: '', drops: 0, html: 0, inpage: 'none', dragend: 'none' };
const render = () => { out.textContent = Object.entries(state).map(([k, v]) => `${k}=${v}`).join('\n'); };
pf.on('drop', async (d) => {
  state.drops++;
  const at = document.elementFromPoint(d.x, d.y);
  const zone = (at && at.closest('[data-zone]')) ? at.closest('[data-zone]').dataset.zone : 'none';
  const files = d.files.map((f) => `${f.name}:${f.size}`).join(',') || 'none';
  state.drop = `files=${files} text=${d.text === null ? 'none' : d.text} url=${d.url || 'none'} zone=${zone} row=${d.row}`;
  state.text0 = '';
  render();
  if (d.files.length) {
    try { state.text0 = await pf.drop.text(0); } catch (e) { state.text0 = 'refused ' + e.message; }
    render();
  }
});
addEventListener('dragover', (e) => e.preventDefault());
addEventListener('drop', (e) => {
  e.preventDefault();
  const card = e.dataTransfer.getData('application/x-card');
  if (card) state.inpage = `${card} on ${e.target.closest('[data-zone]') ? e.target.closest('[data-zone]').dataset.zone : 'none'}`;
  else state.html++;
  render();
});
document.getElementById('card').addEventListener('dragstart', (e) => e.dataTransfer.setData('application/x-card', 'c1'));
document.getElementById('txt').addEventListener('dragstart', (e) => e.dataTransfer.setData('text/plain', 'text from the page'));
addEventListener('dragend', (e) => { state.dragend = `${e.target.id}:${e.dataTransfer.dropEffect}`; render(); });
render();
