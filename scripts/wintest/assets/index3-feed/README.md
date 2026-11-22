A plugin feed as the plugin repository's `tools/build-index.py` publishes it once plugin interface 3
exists: `index-3.json` with the whole entries and `index.json` for PecoFence 0.5.0, where both
plugins (`folders: "any"`, which 0.5.0 cannot read) are stand-ins with `api: 3` and pf-keep keeps its
take-down. Made with the build-index of the plugin repository's branch `plugin-limits`
(`for_0_5`, `entry`); read by `tests/plugin-index3.ps1`.
