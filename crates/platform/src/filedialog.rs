//! Common file dialogs (`IFileOpenDialog` / `IFileSaveDialog`) for config import / export.

use crate::bindings::*;
use crate::wide::to_wide;
use std::path::PathBuf;
use windows_core::{Interface, PCWSTR, Result};

fn result_path(dialog: &IFileDialog) -> Result<PathBuf> {
    // SAFETY: COM calls on a live dialog after a successful Show.
    unsafe {
        let item = dialog.GetResult()?;
        let p = item.GetDisplayName(SIGDN_FILESYSPATH)?;
        let s = p.to_string().unwrap_or_default();
        CoTaskMemFree(p.0.cast());
        Ok(PathBuf::from(s))
    }
}

/// "Save as" for a JSON file. `None` when the user cancelled.
pub fn save_json(
    owner: Option<HWND>,
    title: &str,
    suggested_name: &str,
) -> Result<Option<PathBuf>> {
    let title_w = to_wide(title);
    let name_w = to_wide(suggested_name);
    let filter_name = to_wide(pecofence_core::i18n::text("JSON 文件 (*.json)"));
    let filter_spec = to_wide("*.json");
    let ext = to_wide("json");
    // SAFETY: COM object created here; all strings outlive the calls.
    unsafe {
        let dialog: IFileSaveDialog = CoCreateInstance(&FileSaveDialog, None, CLSCTX_ALL as u32)?;
        let base: IFileDialog = dialog.cast()?;
        base.SetOptions(FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST | FOS_OVERWRITEPROMPT)
            .ok()?;
        let spec = COMDLG_FILTERSPEC {
            pszName: PCWSTR(filter_name.as_ptr()),
            pszSpec: PCWSTR(filter_spec.as_ptr()),
        };
        base.SetFileTypes(1, &spec).ok()?;
        base.SetDefaultExtension(PCWSTR(ext.as_ptr())).ok()?;
        base.SetTitle(PCWSTR(title_w.as_ptr())).ok()?;
        base.SetFileName(PCWSTR(name_w.as_ptr())).ok()?;
        let hr = base.Show(owner);
        if hr.is_err() {
            // ERROR_CANCELLED and friends: not an error for the caller.
            return Ok(None);
        }
        result_path(&base).map(Some)
    }
}

/// "Open" for a JSON file. `None` when the user cancelled.
pub fn open_json(owner: Option<HWND>, title: &str) -> Result<Option<PathBuf>> {
    let title_w = to_wide(title);
    let filter_name = to_wide(pecofence_core::i18n::text("JSON 文件 (*.json)"));
    let filter_spec = to_wide("*.json");
    // SAFETY: COM object created here; all strings outlive the calls.
    unsafe {
        let dialog: IFileOpenDialog = CoCreateInstance(&FileOpenDialog, None, CLSCTX_ALL as u32)?;
        let base: IFileDialog = dialog.cast()?;
        base.SetOptions(FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST | FOS_FILEMUSTEXIST)
            .ok()?;
        let spec = COMDLG_FILTERSPEC {
            pszName: PCWSTR(filter_name.as_ptr()),
            pszSpec: PCWSTR(filter_spec.as_ptr()),
        };
        base.SetFileTypes(1, &spec).ok()?;
        base.SetTitle(PCWSTR(title_w.as_ptr())).ok()?;
        let hr = base.Show(owner);
        if hr.is_err() {
            return Ok(None);
        }
        result_path(&base).map(Some)
    }
}
