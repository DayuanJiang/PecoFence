A plugin feed as the plugin repository's `tools/build-index.py` publishes it once plugin interface 3
exists: `index-3.json` with the whole entries and `index.json` for PecoFence 0.5.0, where the four
plugins are stand-ins (`standIn`, `api: 3`) and pf-keep keeps its take-down. pf-big's 2 MB
`data.json` is not kept here: `tests/plugin-index3.ps1` writes it again byte for byte. Made with
the build-index of the plugin repository's branch `plugin-limits` (`for_0_5`, `entry`).
