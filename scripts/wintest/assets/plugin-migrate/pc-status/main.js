// Stand-in for the pc-status plugin: shows that it runs and the settings its fence has.
pf.on("start", () => pf.show({ rows: [{ text: "pc-status stand-in, settings " + JSON.stringify(pf.settings) }] }));
