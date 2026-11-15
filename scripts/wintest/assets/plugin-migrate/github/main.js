// Stand-in for the github plugin: shows that it runs and the settings its fence has.
pf.on("start", () => pf.show({ rows: [{ text: "github stand-in, settings " + JSON.stringify(pf.settings) }] }));
