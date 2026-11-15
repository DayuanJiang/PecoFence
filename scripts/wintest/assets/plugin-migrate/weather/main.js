// Stand-in for the weather plugin: shows that it runs and the settings its fence has.
pf.on("start", () => pf.show({ rows: [{ text: "weather stand-in, settings " + JSON.stringify(pf.settings) }] }));
