//! Notices in the corner (`crate::notice`, 30 s each) that show what PecoFence can do:
//! - the first start shows a few tips one after another (making, hiding and peeking at fences,
//!   tabs); 「下一条」 shows the next, the last one's button opens the online manual (also on
//!   Settings › 「关于」), × or the timeout ends them;
//! - a start after an update to a release with a what's-new notice shows that once; its button
//!   opens Settings › 「小工具」 at the 对话框跳转 demo;
//! - while 对话框跳转 is off, the first file dialog that comes up while File Explorer shows
//!   another folder is offered a jump there (`dialog_jump::set_offering`), once; taking it jumps
//!   the dialog and switches the tool on.
//!
//! None of them shows under a test script or `--exit-after`. What was shown is kept in
//! `notices.json` beside config.json.

use super::*;

/// Kept beside config.json, so each test instance has its own.
const FILE: &str = "notices.json";
/// The what's-new notice of this build: bumped when a release brings one of its own
/// (1: 0.3.0). A first start records it as seen, so new users get the tips instead.
const WHATS_NEW: u32 = 1;
/// The settings page element the what's-new and 对话框跳转 notices scroll to.
const DIALOG_JUMP_DEMO: &str = "dialogJumpDemo";

#[derive(Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct Saved {
    whats_new: u32,
    /// The 对话框跳转 offer was shown.
    dialog_jump_offered: bool,
}

/// The notice in `App::welcome`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Welcome {
    /// The first start's tip at this index of `App::tips`.
    Tip(usize),
    WhatsNew,
    /// 对话框跳转 offered for this dialog, to go to this folder.
    DialogJump {
        dialog: isize,
        dir: PathBuf,
    },
    /// 对话框跳转 was switched on by the offer.
    DialogJumpOn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Tip {
    Create,
    Hide,
    Peek,
    Tabs,
}

impl App {
    fn notices_path(&self) -> PathBuf {
        self.state
            .config_path()
            .parent()
            .unwrap_or(Path::new("."))
            .join(FILE)
    }

    fn load_notices(&self) -> Saved {
        std::fs::read(self.notices_path())
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Saved>(&bytes).ok())
            .unwrap_or_default()
    }

    fn save_notices(&self, saved: &Saved) {
        let path = self.notices_path();
        // The first start records itself before the first configuration save made the folder.
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Err(error) = serde_json::to_vec_pretty(saved)
            .map_err(std::io::Error::other)
            .and_then(|j| std::fs::write(&path, j))
        {
            tracing::warn!(%error, "notice state not saved");
        }
    }

    /// At startup: the tips on a first start, else the what's-new notice if this build has one
    /// the user has not seen. Under a test script only the record is kept.
    pub(super) fn offer_welcome(&mut self) {
        let mut saved = self.load_notices();
        let first_run = self.state.first_run;
        let offer = if first_run {
            Some(Welcome::Tip(0))
        } else if saved.whats_new < WHATS_NEW {
            Some(Welcome::WhatsNew)
        } else {
            None
        };
        tracing::info!(
            first_run,
            seen = saved.whats_new,
            current = WHATS_NEW,
            show = self.notices,
            offer = ?offer,
            "welcome notices"
        );
        if saved.whats_new < WHATS_NEW {
            // Once, whether or not it is clicked.
            saved.whats_new = WHATS_NEW;
            self.save_notices(&saved);
        }
        if self.notices
            && let Some(offer) = offer
        {
            self.show_welcome(offer);
        }
    }

    /// The 对话框跳转 offer watches while the tool is off and was never offered.
    pub(super) fn sync_dialog_jump_offer(&self) {
        let on = self.notices
            && !self.state.config.settings.tools.dialog_jump
            && !self.load_notices().dialog_jump_offered;
        crate::dialog_jump::set_offering(on, self.control.hwnd());
    }

    /// `WM_APP_DIALOG_OFFER`: a file dialog to offer 对话框跳转 to.
    pub(super) fn offer_dialog_jump(&mut self) {
        let Some((dialog, dir)) = crate::dialog_jump::take_offer() else {
            return;
        };
        let mut saved = self.load_notices();
        if self.state.config.settings.tools.dialog_jump || saved.dialog_jump_offered {
            return;
        }
        if self.notice_showing() {
            // A later dialog gets the offer.
            tracing::info!("dialog jump offer: another notice is showing; not now");
            return;
        }
        self.show_welcome(Welcome::DialogJump {
            dialog: dialog.0 as isize,
            dir,
        });
        if self.welcome.is_some() {
            saved.dialog_jump_offered = true;
            self.save_notices(&saved);
            self.sync_dialog_jump_offer();
        }
    }

    /// The first start's tips: Peek's only while a shortcut opens it.
    fn tips(&self) -> Vec<Tip> {
        let peek = &self.state.config.settings.peek;
        let peek = self.peek_hotkey.is_some() || (peek.enabled && peek.double_ctrl);
        [Tip::Create, Tip::Hide, Tip::Peek, Tip::Tabs]
            .into_iter()
            .filter(|t| *t != Tip::Peek || peek)
            .collect()
    }

    fn show_welcome(&mut self, which: Welcome) {
        use pecofence_core::i18n::{format, text};
        let (title, body, action) = match &which {
            Welcome::WhatsNew => (
                format("PecoFence {0} 的新功能", &["0.3.0".to_string()]),
                text(
                    "· 小工具：便签和待办、时钟、脚本栅栏\n· 速览时直接打字搜索，支持拼音\n· 「打开/保存」对话框跳到刚打开的文件夹\n小工具和对话框跳转默认关着，在设置里打开。",
                )
                .to_string(),
                text("打开小工具设置"),
            ),
            Welcome::DialogJump { dir, .. } => {
                let name = dir.file_name().map_or_else(
                    || dir.display().to_string(),
                    |n| n.to_string_lossy().into_owned(),
                );
                (
                    format("让对话框跳到「{0}」？", std::slice::from_ref(&name)),
                    format(
                        "您刚在资源管理器里打开了「{0}」。PecoFence 可以让这个对话框跳过去，以后从资源管理器切回对话框时也自动跳。",
                        &[name],
                    ),
                    text("跳过去并开启"),
                )
            }
            Welcome::DialogJumpOn => (
                text("对话框跳转已开启").to_string(),
                text(
                    "以后从资源管理器切回「打开/保存」对话框，它会自动跳到那个文件夹；按 Ctrl+G 也能跳。不需要的话，可以在「设置」→「小工具」里关掉。",
                )
                .to_string(),
                text("设置…"),
            ),
            Welcome::Tip(i) => {
                let i = *i;
                let tips = self.tips();
                let Some(&tip) = tips.get(i) else {
                    return;
                };
                let (name, body) = match tip {
                    Tip::Create => (
                        text("新建栅栏"),
                        text(
                            "在桌面空白处按住鼠标拖出一个框，松开后选「在此新建栅栏」。文件都没有移动，只是显示在栅栏里。",
                        )
                        .to_string(),
                    ),
                    Tip::Hide => (
                        text("隐藏栅栏"),
                        text("双击桌面空白处，所有栅栏收起来；再双击一次，栅栏就回来。")
                            .to_string(),
                    ),
                    Tip::Peek => (
                        text("速览和搜索"),
                        match self.peek_hotkey {
                            Some(h) => format(
                                "按 {0}，栅栏浮到所有窗口前面。这时直接打字，就能搜索栅栏里的图标，按 Enter 打开。",
                                &[super::peek::peek_hotkey_label(h)],
                            ),
                            None => text(
                                "连按两下 Ctrl，栅栏浮到所有窗口前面。这时直接打字，就能搜索栅栏里的图标，按 Enter 打开。",
                            )
                            .to_string(),
                        },
                    ),
                    Tip::Tabs => (
                        text("合成标签页"),
                        text("把一个栅栏拖到另一个栅栏的标题上，两个栅栏就合成一个，用标签切换。")
                            .to_string(),
                    ),
                };
                let title = format(
                    "小技巧 {0}/{1}：{2}",
                    &[
                        (i + 1).to_string(),
                        tips.len().to_string(),
                        name.to_string(),
                    ],
                );
                let last = i + 1 == tips.len();
                let action = if last {
                    text("使用手册")
                } else {
                    text("下一条")
                };
                (title, body, action)
            }
        };
        let kind = welcome_kind(&which);
        match crate::notice::Notice::show(&self.ctx, &title, &body, action, crate::notice::locale())
        {
            Ok(notice) => {
                tracing::info!(notice = %kind, "welcome notice shown");
                self.welcome = Some((notice, which));
            }
            Err(error) => tracing::warn!(%error, notice = %kind, "welcome notice not shown"),
        }
    }

    /// The welcome notice `which` was clicked, closed or timed out.
    pub(super) fn on_welcome_notice(&mut self, which: Welcome, event: crate::notice::NoticeEvent) {
        tracing::info!(notice = %welcome_kind(&which), ?event, "welcome notice answered");
        if event != crate::notice::NoticeEvent::Action {
            return;
        }
        match which {
            Welcome::Tip(i) if i + 1 < self.tips().len() => self.show_welcome(Welcome::Tip(i + 1)),
            Welcome::Tip(_) => self.open_manual(),
            Welcome::WhatsNew | Welcome::DialogJumpOn => {
                self.open_settings_page("tools", Some(DIALOG_JUMP_DEMO))
            }
            Welcome::DialogJump { dialog, dir } => {
                let mut new = self.state.config.settings.clone();
                new.tools.dialog_jump = true;
                self.apply_settings(new, "dialog jump offer");
                crate::dialog_jump::jump_to(HWND(dialog as *mut core::ffi::c_void), dir);
                self.show_welcome(Welcome::DialogJumpOn);
            }
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
        tracing::info!(%url, "opening the manual");
        let _ = shell::shell_execute(Path::new(&url), None, None);
    }
}

/// The notice's kind for the log (without the folder of an offer).
fn welcome_kind(which: &Welcome) -> String {
    match which {
        Welcome::Tip(i) => format!("Tip({i})"),
        Welcome::WhatsNew => "WhatsNew".into(),
        Welcome::DialogJump { dialog, .. } => format!("DialogJump(dialog={dialog})"),
        Welcome::DialogJumpOn => "DialogJumpOn".into(),
    }
}
