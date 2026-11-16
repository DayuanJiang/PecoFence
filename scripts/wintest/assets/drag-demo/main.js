// The test plugin of tests/plugin-drag.ps1: rows that drag a data file, a file of a folder the
// user picked, a text and links out, rows whose drag PecoFence refuses, and on the last rows what
// came back (dragend, and a drop with the row it landed on).
let ended = 'none', dropped = 'none', ends = 0;
const show = () => pf.show({
  rows: [
    { id: 'pic', text: 'drag the picture', drag: { file: 'pic.png' } },
    { id: 'text', text: 'drag a text', drag: { text: 'text from the plugin' } },
    { id: 'link', text: 'drag a link', drag: { url: 'https://example.com/page' } },
    { id: 'picked', text: 'drag a picked file', drag: { file: 'C:\\pf\\run\\drag\\picked.txt' } },
    { id: 'other', text: 'refused: a link elsewhere', drag: { url: 'https://other.org/' } },
    { id: 'hta', text: 'refused: a program', drag: { file: 'run.hta' } },
    { id: 'outside', text: 'refused: a file outside', drag: { file: 'C:\\pf\\run\\outside.txt' } },
    { id: 'end', text: `dragend ${ended} ends=${ends}` },
    { id: 'drop', text: `drop ${dropped}` },
  ],
});
pf.on('start', show);
pf.on('dragend', (e) => { ends++; ended = `row=${e.row} dropped=${e.dropped}`; show(); });
pf.on('drop', (d) => { dropped = `row=${d.row} text=${d.text === null ? 'none' : d.text} files=${d.files.length}`; show(); });
