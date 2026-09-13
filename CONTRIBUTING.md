# Contributing

Start with [development setup](docs/DEVELOPMENT.md). Keep changes focused and
describe the behavior being fixed or added.

Before submitting a change:

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python scripts/check-locales.py
python scripts/check-readme-translations.py
```

For settings changes, run `node scripts/test-settings-ui.mjs`, open the local
test page and wait for its pass/fail summary. Include the result and the Windows
build/DPI used for any native UI verification.

Translation contributions are welcome. See [LOCALIZATION.md](docs/LOCALIZATION.md)
for catalog keys, placeholders and adding languages. Preserve user-provided text
and do not introduce network requests for translation.

Please keep build outputs, local configuration, crash dumps, recordings and
credentials out of commits. The optional `extras/pecofence-promo` project has its
own Node dependencies; the desktop application does not need Node to build.
