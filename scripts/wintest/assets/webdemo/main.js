// The test web plugin's main.js (QuickJS, beside the page): answers each message of the page.
pf.on("message", (m) => pf.post({ echo: m, from: "main.js" }));
