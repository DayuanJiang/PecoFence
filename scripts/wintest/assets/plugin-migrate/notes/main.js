// Stand-in for the notes plugin: shows that it runs and the settings its fence has.
pf.on("start", () => pf.show({ rows: [{ text: "notes stand-in, settings " + JSON.stringify(pf.settings) }] }));
