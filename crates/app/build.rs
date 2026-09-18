// Embeds the application icon and a VERSIONINFO resource into the executable.
//
// No build-time crate dependencies: a small `.rc` is written to `OUT_DIR`, compiled with the
// Windows SDK's `rc.exe` and handed to the linker. If `rc.exe` cannot be found the build still
// succeeds, without an icon (a `cargo:warning` says so). `PECOFENCE_RC` overrides the path.
//
// `crates/watchdog/build.rs` includes this file so both executables carry the same icon.

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    // Shared icon; the path is the same relative to either crate directory.
    let ico = manifest_dir
        .join("..")
        .join("app")
        .join("assets")
        .join("pecofence.ico");
    let ico = ico.canonicalize().unwrap_or(ico);

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../app/build.rs");
    println!("cargo:rerun-if-changed={}", ico.display());
    println!("cargo:rerun-if-env-changed=PECOFENCE_RC");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows")
        || env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("msvc")
    {
        return;
    }
    let Some(rc) = find_rc() else {
        println!("cargo:warning=rc.exe not found (Windows SDK); building without an embedded icon");
        return;
    };

    let rc_path = out_dir.join("pecofence.rc");
    std::fs::write(&rc_path, rc_source(&ico)).expect("write .rc");
    let res_path = out_dir.join("pecofence.res");
    // /nologo, /fo output; no /i needed because the script includes no headers.
    let status = Command::new(&rc)
        .arg("/nologo")
        .arg("/fo")
        .arg(&res_path)
        .arg(&rc_path)
        .status();
    match status {
        Ok(s) if s.success() => {
            println!("cargo:rustc-link-arg-bins={}", res_path.display());
        }
        Ok(s) => println!(
            "cargo:warning={} failed ({s}); building without an embedded icon",
            rc.display()
        ),
        Err(e) => println!(
            "cargo:warning=could not run {}: {e}; building without an embedded icon",
            rc.display()
        ),
    }
}

/// The resource script: icon 1 plus a VERSIONINFO block derived from the Cargo manifest.
fn rc_source(ico: &Path) -> String {
    let ver = |k: &str| env::var(format!("CARGO_PKG_VERSION_{k}")).unwrap_or_else(|_| "0".into());
    let (major, minor, patch) = (ver("MAJOR"), ver("MINOR"), ver("PATCH"));
    let version = env::var("CARGO_PKG_VERSION").unwrap_or_default();
    let name = env::var("CARGO_PKG_NAME").unwrap_or_default();
    let description = env::var("CARGO_PKG_DESCRIPTION").unwrap_or_default();
    let description = if description.is_empty() {
        name.clone()
    } else {
        description
    };
    // rc string literals: backslashes must be doubled.
    let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\"\"");
    let ico = esc(ico.display().to_string().trim_start_matches(r"\\?\"));
    format!(
        r#"1 ICON "{ico}"

1 VERSIONINFO
FILEVERSION {major},{minor},{patch},0
PRODUCTVERSION {major},{minor},{patch},0
FILEOS 0x40004L
FILETYPE 0x1L
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "040904b0"
    BEGIN
      VALUE "CompanyName", "Dayuan Jiang"
      VALUE "FileDescription", "{description}"
      VALUE "FileVersion", "{version}"
      VALUE "InternalName", "{name}"
      VALUE "LegalCopyright", "Copyright (c) Dayuan Jiang. Licensed under the Apache License 2.0."
      VALUE "OriginalFilename", "{name}.exe"
      VALUE "ProductName", "PecoFence"
      VALUE "ProductVersion", "{version}"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x409, 1200
  END
END
"#,
        description = esc(&description),
        name = esc(&name),
        version = esc(&version),
    )
}

/// `PECOFENCE_RC`, else the newest `rc.exe` under the Windows 10/11 SDK, else `rc.exe` on PATH.
fn find_rc() -> Option<PathBuf> {
    if let Some(p) = env::var("PECOFENCE_RC").ok().filter(|p| !p.is_empty()) {
        let p = PathBuf::from(p);
        return p.is_file().then_some(p);
    }
    let kits = env::var("ProgramFiles(x86)")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(r"C:\Program Files (x86)"))
        .join(r"Windows Kits\10\bin");
    let mut best: Option<(Vec<u32>, PathBuf)> = None;
    if let Ok(entries) = std::fs::read_dir(&kits) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            if !name.starts_with("10.") {
                continue;
            }
            let key: Vec<u32> = name.split('.').map(|p| p.parse().unwrap_or(0)).collect();
            let candidate = entry.path().join("x64").join("rc.exe");
            if candidate.is_file() && best.as_ref().is_none_or(|(k, _)| key > *k) {
                best = Some((key, candidate));
            }
        }
    }
    if let Some((_, path)) = best {
        return Some(path);
    }
    // Fall back to PATH (e.g. inside a Developer Command Prompt).
    Command::new("rc.exe")
        .arg("/?")
        .output()
        .is_ok()
        .then(|| PathBuf::from("rc.exe"))
}
