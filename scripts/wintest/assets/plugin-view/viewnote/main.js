// The note test plugin of plugin-view.ps1: the fence is a note that PecoFence edits and keeps;
// the row under it (not drawn: the note takes the fence) says what text the plugin got last.
let got = "-";

function show() {
  pf.show({ rows: [{ input: { id: "note", type: "note" } }, { text: "got: " + got }] });
}

pf.on("start", show);
pf.on("text", (e) => {
  got = e.id + " " + e.text.replace(/\n/g, "|");
  show();
});
