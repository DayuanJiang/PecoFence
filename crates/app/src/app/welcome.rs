//! The first start's welcome: a notice in the corner (`crate::notice`, 30 s) says the files
//! stay where they are and how to make and hide fences, and its button opens the online
//! manual (also on Settings › 「关于」).

use super::*;

impl App {
    pub(super) fn offer_welcome(&mut self) {
        match crate::notice::Notice::show(
            &self.ctx,
            pecofence_core::i18n::text("桌面整理好了"),
            pecofence_core::i18n::text(
                "文件都没有移动，只是显示在栅栏里。\n在桌面空白处拖出一个框可新建栅栏，双击空白处可隐藏或显示栅栏。",
            ),
            pecofence_core::i18n::text("使用手册"),
            crate::notice::locale(),
        ) {
            Ok(notice) => self.welcome = Some(notice),
            Err(error) => tracing::warn!(%error, "welcome notice not shown"),
        }
    }

    /// The manual in the UI language; English lives at the site root.
    pub(super) fn open_manual(&self) {
        let tag = pecofence_core::i18n::language().tag();
        let dir = if tag == "en" {
            String::new()
        } else {
            format!("{tag}/")
        };
        let url = format!("https://pecofence.jiang.jp/{dir}manual/");
        let _ = shell::shell_execute(Path::new(&url), None, None);
    }
}
