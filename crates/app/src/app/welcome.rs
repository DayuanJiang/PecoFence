//! Notices in the corner (`crate::notice`, 30 s each) that show what PecoFence can do:
//! - the first start shows a few tips one after another (making, hiding and peeking at fences,
//!   tabs); 「下一条」 shows the next, the last one's button opens the online manual (also on
//!   Settings › 「关于」), × or the timeout ends them;
//! - a start after an update to a release with a what's-new notice shows that once; its button
//!   opens Settings › 「小工具」 at the 对话框跳转 demo;
//! - while 对话框跳转 is off, the first file dialog that comes up while File Explorer shows
//!   another folder is offered a jump there (`dialog_jump::set_offering`), once; taking it jumps
//!   the dialog and switches the tool on;
//! - 功能提示 (Settings › 常规, `featureTips`): three cards, each once ever, when a feature
//!   would help right then: 收起 once the user's new fence makes four fences besides 「桌面」;
//!   在桌面显示此文件夹 when a folder in a fence is opened for the third time; a rule after
//!   three files of one extension were dragged into one fence in a run. A card that cannot show
//!   (another card is up) waits for the next such moment; the two offers take the place of the
//!   收起 card, and that card waits until the name box of the fence just made has closed.
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
    /// 功能提示 shown: 收起, 在桌面显示此文件夹, the rule offer.
    roll_tip_shown: bool,
    folder_tip_shown: bool,
    rule_tip_shown: bool,
    /// The version whose update card the user saw (it is not shown again).
    update_card: String,
    /// The version whose card showed while nobody was there, and when (unix seconds): it is
    /// offered once more a day later.
    update_card_unseen: String,
    update_card_unseen_at: i64,
}

/// What the update card did for which version (kept in `notices.json`; the app keeps a copy
/// in memory, so a failed write cannot bring the card back in the same run).
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct UpdateCardRecord {
    pub seen: String,
    pub unseen: String,
    pub unseen_at: i64,
}

/// 功能提示: the roll-up tip shows once this many fences with their own window hold items
/// (「桌面」, tabs and tool fences not counted; a first start makes three).
const ROLL_TIP_FENCES: usize = 4;
/// 功能提示: opens of one folder item before the folder tip.
pub(super) const FOLDER_TIP_OPENS: u32 = 3;
/// 功能提示: files of one extension dragged into one fence before the rule tip.
const RULE_TIP_FILES: u32 = 3;

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
    /// 功能提示: fences roll up.
    RollTip,
    /// 功能提示: `dir`, opened from `fence` again and again (`name` as the fence shows it), can
    /// be shown on the desktop.
    FolderTip {
        fence: FenceId,
        dir: PathBuf,
        name: String,
    },
    /// 功能提示: a rule for `ext` files (`.pdf`) into `fence`, where `count` of them were dragged.
    RuleTip {
        fence: FenceId,
        ext: String,
        count: u32,
    },
    /// The item menu's 「以后把 .pdf 文件都放到这里」 made (`already`: found) the rule that
    /// sends files ending in `ext` to the fence named `fence`.
    RuleAdded {
        ext: String,
        fence: String,
        already: bool,
    },
}

/// What the roll-up tip waits for after a fence was made with its name box
/// (`App::roll_tip_waits`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum RollTipWait {
    #[default]
    No,
    /// The fence fades in; its name box opens after that.
    Fade,
    /// The name box is open.
    NameBox,
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

    pub(super) fn load_update_card_record(&self) -> UpdateCardRecord {
        let saved = self.load_notices();
        UpdateCardRecord {
            seen: saved.update_card,
            unseen: saved.update_card_unseen,
            unseen_at: saved.update_card_unseen_at,
        }
    }

    pub(super) fn save_update_card_record(&self, record: &UpdateCardRecord) {
        let mut saved = self.load_notices();
        saved.update_card = record.seen.clone();
        saved.update_card_unseen = record.unseen.clone();
        saved.update_card_unseen_at = record.unseen_at;
        self.save_notices(&saved);
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

    /// Title, body and button of the card `which` (None: a tip that does not exist).
    fn welcome_texts(&self, which: &Welcome) -> Option<(String, String, &'static str)> {
        use pecofence_core::i18n::{format, text};
        Some(match which {
            Welcome::WhatsNew => (
                format("PecoFence {0} 的新功能", &["0.3.0".to_string()]),
                text(
                    "· 速览时直接打字搜索，支持拼音\n· “打开/保存”对话框能跳到资源管理器里正开着的文件夹\n· 小工具：便签和待办、时钟、脚本栅栏\n对话框跳转要在设置里开启。",
                )
                .to_string(),
                text("打开设置"),
            ),
            Welcome::DialogJump { dir, .. } => {
                let name = dir.file_name().map_or_else(
                    || dir.display().to_string(),
                    |n| n.to_string_lossy().into_owned(),
                );
                (
                    format("让对话框跳到“{0}”？", std::slice::from_ref(&name)),
                    format(
                        "资源管理器里正开着“{0}”。PecoFence 可以让这个对话框跳过去，以后从资源管理器切回对话框时也会自动跳。",
                        &[name],
                    ),
                    text("跳过去并开启"),
                )
            }
            Welcome::DialogJumpOn => (
                text("对话框跳转已开启").to_string(),
                text(
                    "以后从资源管理器切回“打开/保存”对话框，对话框会自动跳到资源管理器里开着的文件夹。按 Ctrl+G 也能跳。不需要的话，可以在“设置”→“速览与搜索”里关掉。",
                )
                .to_string(),
                text("设置…"),
            ),
            Welcome::RollTip => {
                let roll = &self.state.config.settings.roll_up;
                let body = if roll.click_to_expand {
                    text("双击标题就只留一行标题，单击标题会展开。")
                } else if roll.hover_peek {
                    text("双击标题就只留一行标题，鼠标停在标题上会自动展开。")
                } else {
                    text("双击标题就只留一行标题，再双击一次就展开。")
                };
                (
                    text("栅栏多了可以收起").to_string(),
                    body.to_string(),
                    text("知道了"),
                )
            }
            Welcome::FolderTip { name, .. } => (
                text("把常用的文件夹放到桌面上").to_string(),
                format(
                    "你常从栅栏打开“{0}”。右键它 → 在桌面显示此文件夹，就能直接在桌面上浏览里面的文件。",
                    std::slice::from_ref(name),
                ),
                text("试一试"),
            ),
            Welcome::RuleTip { fence, ext, count } => {
                let title = self
                    .state
                    .fence(*fence)
                    .map(|f| self.state.display_title(f))
                    .unwrap_or_default();
                (
                    text("让新文件自动进来").to_string(),
                    format(
                        "你把 {0} 个 {1} 文件拖进了“{2}”。以后新的 {1} 文件也自动放到这里吗？",
                        &[count.to_string(), ext.clone(), title],
                    ),
                    text("添加规则"),
                )
            }
            Welcome::RuleAdded {
                ext,
                fence,
                already,
            } => (
                if *already {
                    text("规则已经存在").to_string()
                } else {
                    text("已添加整理规则").to_string()
                },
                if self.state.config.rules.keep_updated {
                    format(
                        "以后桌面上新出现的 {0} 文件会自动放进“{1}”。要修改或删除这条规则，打开“设置”→“整理规则”。",
                        &[ext.clone(), fence.clone()],
                    )
                } else {
                    format(
                        "“新项目出现在桌面时自动归类”现在是关闭的：打开它，或从托盘菜单选“立即应用整理规则”，{0} 文件才会放进“{1}”。规则在“设置”→“整理规则”里修改。",
                        &[ext.clone(), fence.clone()],
                    )
                },
                text("打开整理规则"),
            ),
            Welcome::Tip(i) => {
                let i = *i;
                let tips = self.tips();
                let &tip = tips.get(i)?;
                let (name, body) = match tip {
                    Tip::Create => (
                        text("新建栅栏"),
                        text(
                            "在桌面空白处按住鼠标拖出一个框，松开后点框右下角的“新建栅栏”。文件还在原来的位置。",
                        )
                        .to_string(),
                    ),
                    Tip::Hide => (
                        text("隐藏栅栏"),
                        text("双击桌面空白处，所有栅栏都会隐藏。再双击一次，栅栏就回来。")
                            .to_string(),
                    ),
                    Tip::Peek => (
                        text("速览和搜索"),
                        match self.peek_hotkey {
                            Some(h) => format(
                                "按 {0}，所有栅栏会显示在窗口前面。这时直接打字，就能搜索栅栏里的图标，按 Enter 打开。",
                                &[super::peek::peek_hotkey_label(h)],
                            ),
                            None => text(
                                "快速按两下 Ctrl，所有栅栏会显示在窗口前面。这时直接打字，就能搜索栅栏里的图标，按 Enter 打开。",
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
        })
    }

    /// The tip that follows when card `which` got `event` (「下一条」 on a tip that is not the
    /// last one).
    pub(super) fn next_tip(
        &self,
        which: &Welcome,
        event: crate::notice::NoticeEvent,
    ) -> Option<usize> {
        match which {
            Welcome::Tip(i)
                if event == crate::notice::NoticeEvent::Action && i + 1 < self.tips().len() =>
            {
                Some(i + 1)
            }
            _ => None,
        }
    }

    /// 「下一条」 on a tip card: the card stays where it is and takes tip `next`'s texts. A
    /// card that cannot take them goes and a new one comes, as before.
    pub(super) fn show_next_tip(&mut self, notice: crate::notice::Notice, next: usize) {
        let which = Welcome::Tip(next);
        let swapped = self
            .welcome_texts(&which)
            .is_some_and(|(title, body, action)| {
                notice
                    .set_content(&title, &body, action, crate::notice::locale())
                    .map_err(|error| tracing::warn!(%error, "tip card did not take the next tip"))
                    .is_ok()
            });
        tracing::info!(notice = %welcome_kind(&which), swapped, "next tip on the same card");
        if swapped {
            self.welcome = Some((notice, which));
        } else {
            // Gone before the new card registers its window class.
            drop(notice);
            self.show_welcome(which);
        }
    }

    pub(super) fn show_welcome(&mut self, which: Welcome) {
        use pecofence_core::i18n::text;
        let Some((title, body, action)) = self.welcome_texts(&which) else {
            return;
        };
        let kind = welcome_kind(&which);
        // The two offers can be turned down; the roll-up tip only informs.
        let secondary = matches!(which, Welcome::FolderTip { .. } | Welcome::RuleTip { .. })
            .then(|| text("不用"));
        let locale = crate::notice::locale();
        // The card whose button brought this one up takes the new texts where it stands: the
        // old card went and the new one faded in from nothing, a blink at the same place. (A
        // card with two buttons is made anew.)
        if let Some(notice) = self.answered_card.take() {
            if secondary.is_none()
                && notice
                    .set_content(&title, &body, action, locale)
                    .map_err(
                        |error| tracing::warn!(%error, "answered card did not take the next card"),
                    )
                    .is_ok()
            {
                tracing::info!(notice = %kind, "welcome notice shown on the card just answered");
                self.welcome = Some((notice, which));
                return;
            }
            notice.leave();
        }
        let avoid = self.notice_avoid_rects();
        let shown = match which {
            // The tips share one card (`show_next_tip`): it is as tall as the tallest of them
            // from the start, so 「下一条」 changes the texts and not the card's size.
            Welcome::Tip(_) => {
                let tallest = (0..self.tips().len())
                    .filter_map(|i| self.welcome_texts(&Welcome::Tip(i)))
                    .filter_map(|(t, b, a)| crate::notice::card_height(&t, &b, a, locale).ok())
                    .fold(0.0, f32::max);
                tracing::debug!(tallest, "tip cards: one height for all");
                crate::notice::Notice::show_reserving(
                    &self.ctx, &title, &body, action, tallest, locale, &avoid,
                )
            }
            _ => crate::notice::Notice::show_with(
                &self.ctx, &title, &body, action, secondary, locale, &avoid,
            ),
        };
        match shown {
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
                self.open_settings_page("peek", Some(DIALOG_JUMP_DEMO))
            }
            Welcome::RuleAdded { .. } => self.open_settings_page("rules", None),
            Welcome::DialogJump { dialog, dir } => {
                let mut new = self.state.config.settings.clone();
                new.tools.dialog_jump = true;
                self.apply_settings(new, "dialog jump offer");
                crate::dialog_jump::jump_to(HWND(dialog as *mut core::ffi::c_void), dir);
                self.show_welcome(Welcome::DialogJumpOn);
            }
            Welcome::RollTip => {}
            Welcome::FolderTip { fence, dir, .. } => {
                // Beside the fence it was opened from, as 在桌面显示此文件夹 places it.
                let (x, y) = self.window_for(fence).map_or_else(
                    || {
                        let p = window::cursor_pos();
                        (p.x, p.y)
                    },
                    |w| {
                        let r = w.rect();
                        (r.right, r.top)
                    },
                );
                self.create_portal(dir, x, y, Some(fence));
            }
            Welcome::RuleTip { fence, ext, .. } => {
                self.add_ext_rule(fence, &ext, self.control.hwnd())
            }
        }
    }

    /// 功能提示 may show a card now: switched on, not a test run, no other card on screen.
    /// An offer for what the user just did (the rule, the folder) takes the place of the
    /// roll-up tip, which only informs: the offer's moment may not come again (three files of
    /// one kind are dragged into a new fence right after it was made, while its tip is up).
    fn feature_tip_allowed(&mut self, what: &'static str) -> bool {
        let on = self.state.config.settings.feature_tips;
        if what != "roll"
            && self.notices
            && on
            && matches!(self.welcome, Some((_, Welcome::RollTip)))
            && let Some((notice, _)) = self.welcome.take()
        {
            tracing::info!(what, "feature tip takes the place of the roll-up tip");
            notice.leave();
        }
        let other_card = self.notice_showing();
        let allowed = self.notices && on && !other_card;
        tracing::info!(
            what,
            notices = self.notices,
            on,
            other_card,
            allowed,
            "feature tip due"
        );
        allowed
    }

    /// After a fence was made with its name box (`create_fence_with_rename`): the roll-up tip
    /// comes when the name has been typed. A card that came up beside the box the moment the
    /// fence appeared took the eye off the name. `after_fade`: the box opens once the fence
    /// has faded in.
    pub(super) fn offer_roll_tip_after_name(&mut self, after_fade: bool) {
        self.roll_tip_waits = if after_fade {
            RollTipWait::Fade
        } else if crate::rename::active_target().is_some() {
            RollTipWait::NameBox
        } else {
            RollTipWait::No
        };
        tracing::debug!(waits = ?self.roll_tip_waits, "roll tip after a new fence");
        if self.roll_tip_waits == RollTipWait::No {
            self.offer_roll_tip();
        }
    }

    /// The new fence has faded in and its name box is open, or it got none (see
    /// `begin_rename_after_fade`): the roll-up tip goes on waiting for the box, or shows.
    pub(super) fn roll_tip_after_fade(&mut self) {
        if self.roll_tip_waits != RollTipWait::Fade {
            return;
        }
        if crate::rename::active_target().is_some() {
            self.roll_tip_waits = RollTipWait::NameBox;
        } else {
            self.roll_tip_waits = RollTipWait::No;
            tracing::debug!("roll tip: the new fence got no name box");
            self.offer_roll_tip();
        }
    }

    /// A name box may have closed (`Command::RenameFence`, or `Command::RedrawAll` for Esc):
    /// the roll-up tip that waited for it shows.
    pub(super) fn roll_tip_after_name_box(&mut self) {
        if self.roll_tip_waits == RollTipWait::NameBox && crate::rename::active_target().is_none() {
            self.roll_tip_waits = RollTipWait::No;
            tracing::debug!("roll tip: the name box of the new fence closed");
            self.offer_roll_tip();
        }
    }

    /// After the user made a fence: the roll-up tip once there are [`ROLL_TIP_FENCES`].
    pub(super) fn offer_roll_tip(&mut self) {
        let fences = self
            .state
            .host_fences()
            .iter()
            .filter(|f| f.tool.is_none() && f.kind != FenceKind::Inbox)
            .count();
        if fences < ROLL_TIP_FENCES {
            return;
        }
        let mut saved = self.load_notices();
        if saved.roll_tip_shown || !self.feature_tip_allowed("roll") {
            return;
        }
        self.show_welcome(Welcome::RollTip);
        if self.welcome.is_some() {
            tracing::info!(fences, "roll tip shown");
            saved.roll_tip_shown = true;
            self.save_notices(&saved);
        }
    }

    /// After a folder item (or a shortcut to a folder) opened in Explorer from `from`: the
    /// folder tip once it was opened [`FOLDER_TIP_OPENS`] times and no folder fence shows it.
    pub(super) fn offer_folder_tip(&mut self, item: ItemId, from: FenceId, dir: &Path) {
        let Some((opens, name)) = self
            .state
            .item(item)
            .map(|it| (it.open_count, it.display_name.clone()))
        else {
            return;
        };
        if opens < FOLDER_TIP_OPENS {
            return;
        }
        let shown = self.state.fences().iter().any(|f| {
            self.state.portal_root(f.id).is_some_and(|p| {
                p.to_string_lossy()
                    .eq_ignore_ascii_case(&dir.to_string_lossy())
            })
        });
        let mut saved = self.load_notices();
        tracing::info!(
            opens,
            shown,
            seen = saved.folder_tip_shown,
            "folder tip: folder opened"
        );
        if shown || saved.folder_tip_shown || !self.feature_tip_allowed("folder") {
            return;
        }
        self.show_welcome(Welcome::FolderTip {
            fence: from,
            dir: dir.to_path_buf(),
            name,
        });
        if self.welcome.is_some() {
            saved.folder_tip_shown = true;
            self.save_notices(&saved);
        }
    }

    /// The fence and file names of a drag and drop that the rule tip counts: a queued
    /// `MoveItems` (from another fence) or `ExternalDrop` (from Explorer). A paste calls the
    /// handler directly and is not counted.
    pub(super) fn tip_drop(&self, cmd: &Command) -> Option<(FenceId, Vec<String>)> {
        if !self.notices || !self.state.config.settings.feature_tips {
            return None;
        }
        let file_name = |p: &Path| p.file_name().map(|n| n.to_string_lossy().into_owned());
        match cmd {
            Command::MoveItems { items, to, .. } => Some((
                *to,
                items
                    .iter()
                    .filter_map(|id| self.state.item(*id))
                    .filter(|it| !it.is_folder && !it.is_namespace())
                    .filter_map(|it| it.path().and_then(|p| file_name(Path::new(p))))
                    .collect(),
            )),
            Command::ExternalDrop { paths, to, mode } if *mode != TransferMode::Link => Some((
                *to,
                paths
                    .iter()
                    .filter(|p| !p.is_dir())
                    .filter_map(|p| file_name(p))
                    .collect(),
            )),
            _ => None,
        }
    }

    /// Files the user dragged into `to` (from another fence or from Explorer): the rule tip
    /// once [`RULE_TIP_FILES`] of one extension went into one fence that the rules do not
    /// already fill with them. Shortcuts and folders are left out: one rule for every `.lnk`
    /// would take all of them.
    pub(super) fn note_tip_drag(&mut self, to: FenceId, names: &[String]) {
        let target_ok = self
            .state
            .fence(to)
            .is_some_and(|f| f.kind == FenceKind::Virtual && f.tool.is_none());
        if !target_ok {
            return;
        }
        let mut due = None;
        for name in names {
            let ext = pecofence_core::rules::ext_of(name);
            if ext.is_empty() || ext == ".lnk" || ext == ".url" {
                continue;
            }
            let facts = pecofence_core::ItemFacts {
                file_name: name.clone(),
                ..Default::default()
            };
            let routed_here = matches!(
                self.state.config.rules.evaluate(&facts),
                pecofence_core::rules::Decision::Route { target: Target::Fence(id), .. } if id == to
            );
            if routed_here {
                continue;
            }
            let count = self.tip_drags.entry((to, ext.clone())).or_default();
            *count += 1;
            if *count >= RULE_TIP_FILES {
                due = Some((ext, *count));
            }
        }
        let Some((ext, count)) = due else {
            return;
        };
        let mut saved = self.load_notices();
        tracing::info!(
            count,
            seen = saved.rule_tip_shown,
            "rule tip: files dragged into a fence"
        );
        if saved.rule_tip_shown || !self.feature_tip_allowed("rule") {
            return;
        }
        self.show_welcome(Welcome::RuleTip {
            fence: to,
            ext,
            count,
        });
        if self.welcome.is_some() {
            saved.rule_tip_shown = true;
            self.save_notices(&saved);
        }
    }

    /// The manual in the UI language; English lives at the site root.
    pub(super) fn open_manual(&self) {
        self.open_manual_at("");
    }

    /// The manual in the UI language at `anchor` (「写法说明」 opens `#write-scripts`).
    pub(super) fn open_manual_at(&self, anchor: &str) {
        let tag = pecofence_core::i18n::language().tag();
        let dir = if tag == "en" {
            String::new()
        } else {
            format!("{tag}/")
        };
        let url = format!("https://pecofence.jiang.jp/{dir}manual/{anchor}");
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
        Welcome::RollTip => "RollTip".into(),
        Welcome::FolderTip { .. } => "FolderTip".into(),
        Welcome::RuleTip { count, .. } => format!("RuleTip(count={count})"),
        Welcome::RuleAdded { already, .. } => format!("RuleAdded(already={already})"),
    }
}
