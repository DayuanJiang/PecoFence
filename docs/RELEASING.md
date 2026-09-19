# Releases

## Local preparation

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python scripts/check-locales.py
python scripts/check-readme-translations.py
./scripts/make-portable.ps1
python scripts/package-source.py
```

The portable ZIP and SHA-256 file appear in `dist/`. The source exporter creates
`dist/github-source/PecoFence/` and a separate source ZIP. It includes existing
tracked and untracked project source while excluding ignored files, internal
engineering archives, local media and Git history.
Curated images and GIFs in `docs/assets/` are included so the root README and the
translations in `docs/readme/` render after upload.

For a first public repository, use the exported source folder if the development
checkout has private screenshots or other artifacts in its historical commits.
Removing a file from the current tree does not remove it from old Git history.

## First GitHub upload

Create an empty GitHub repository under your own account. From the exported
source directory, replace the example remote with its actual URL:

```powershell
git init -b main
git add .
git commit -m "Initial PecoFence release"
git remote add origin https://github.com/YOUR-ACCOUNT/YOUR-REPOSITORY.git
git push -u origin main
```

Use your configured Git identity. Review the exported files before committing.
The project does not assume or claim a particular GitHub organization. After the
first push, set the repository URL in `site/site.json` and follow
[WEBSITE.md](WEBSITE.md) to publish the product page.

## Versioned release

Update the workspace version in `Cargo.toml`, regenerate `Cargo.lock` if needed,
and update `CHANGELOG.md`. Push a matching `v<version>` tag:

```powershell
git tag v0.0.1
git push origin v0.0.1
```

The release workflow checks the tag against `Cargo.toml`, verifies the project,
builds the x64 portable archive and publishes the GitHub Release with its ZIP
and checksum right away (pushing a `v*` tag is the publish action; the winget
workflow then opens the manifest PR). Edit the generated notes afterwards if needed.

Current packaging is x64 and unsigned. ARM64 is not configured. winget publishing runs
from `.github/workflows/winget.yml` on published releases; the Microsoft Store package is
built locally with `./scripts/make-msix.ps1` and uploaded in Partner Center, see
[STORE.md](STORE.md).
