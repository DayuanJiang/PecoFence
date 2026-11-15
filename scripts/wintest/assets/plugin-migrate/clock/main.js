// Stand-in for the clock plugin: shows that it runs and the settings its fence has.
pf.on("start", () => pf.show({ rows: [{ text: "clock stand-in, settings " + JSON.stringify(pf.settings) }] }));
