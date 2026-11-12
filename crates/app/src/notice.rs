//! A notice in the corner above the taskbar, styled like a fence (`pecofence_render::notice`).
//! Windows decides how long a tray balloon stays (5 s by default) and keeps none in the
//! notification centre, so the one-time rating offer (`app/rating.rs`) uses this instead: it
//! stays for [`SHOW_FOR`], longer while the pointer rests on it. It appears without taking
//! focus; a click on it is the user's input to PecoFence, so what the click opens (the Store's
//! rating dialog) may come to the front.

use crate::commands::{Command, CommandQueue};
use crate::fence_window::FenceContext;
use crate::shadow::ShadowWindow;
use pecofence_platform::window::{
    self, ClassOptions, StandardCursor, Window, WindowBuilder, WindowClass, style,
};
use pecofence_platform::{HWND, RECT, desktop, dwm, monitors, msg};
use pecofence_render::motion::{self, Curve, Motion, Tween};
use pecofence_render::notice::{NoticeCard, NoticeHit, NoticeLayout, WIDTH};
use pecofence_render::{ContainerVisual, DesktopWindowTarget, Panel, RenderStack, Theme};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use windows_core::Result;

const CLASS: &str = "PecoFence.Notice";
const TIMER_DISMISS: usize = 1;
/// Steps the 167 ms fade in / out of the card together with its shadow (a layered window).
const TIMER_FADE: usize = 2;
/// How long the notice stays when the pointer leaves it alone.
const SHOW_FOR_MS: u32 = 30_000;
/// Re-check interval while the pointer rests on it.
const HOVER_RECHECK_MS: u32 = 1_500;
/// Gap to the work area's edge, in DIPs.
const MARGIN: f32 = 12.0;

thread_local! {
    /// Cards that were answered and fade out by themselves ([`Notice::leave`]). The next card
    /// drops them as it shows.
    static LEAVING: RefCell<Vec<Notice>> = const { RefCell::new(Vec::new()) };
    /// The cards' window class, registered with the first card: two cards can show at once
    /// (an undo card above a tip).
    static NOTICE_CLASS: RefCell<Option<WindowClass>> = const { RefCell::new(None) };
}

/// The `locale` for [`Notice::show`] in the UI language (fonts for Han, kana, Hangul).
pub fn locale() -> &'static str {
    match pecofence_core::i18n::language().tag() {
        "zh-CN" => "zh-CN",
        "zh-TW" => "zh-TW",
        "ja" => "ja-JP",
        "ko" => "ko-KR",
        _ => "en-US",
    }
}

/// How tall (DIPs) a card with these texts is by itself.
pub fn card_height(title: &str, body: &str, action: &str, locale: &str) -> Result<f32> {
    Ok(NoticeCard::new(title, body, action, locale)?
        .layout()?
        .height)
}

/// What the user did with the notice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoticeEvent {
    /// The card or its button was clicked.
    Action,
    /// Closed with ×, or left alone until it timed out.
    Dismissed,
    /// The plain second button (「不用」) was clicked.
    Declined,
}

struct View {
    panel: Panel,
    stack: Rc<RenderStack>,
    card: NoticeCard,
    layout: NoticeLayout,
    theme: Theme,
    dpi: u32,
    hover: NoticeHit,
    /// The button (or ×) the mouse button went down on, until it comes up or the pointer
    /// leaves the card: it shows pressed while the pointer is over it.
    pressed: NoticeHit,
    root: ContainerVisual,
    motion: Rc<Motion>,
    shadow: ShadowWindow,
    /// Opacity while fading in or out.
    fade: Option<Tween>,
    /// Fading out; the notice no longer reacts.
    closing: bool,
    /// Closed by the second button: the fade's end reports [`NoticeEvent::Declined`].
    declined: bool,
    /// Answered and handed back by the app ([`Notice::leave`]): the fade's end hides the card
    /// and reports nothing.
    left: bool,
    /// Answered and kept by the app for the card the answer may bring up ([`Notice::hold`]):
    /// it takes no more clicks until it has other texts.
    held: bool,
}

impl View {
    /// Fades the card and its shadow to `opacity` from where they are.
    fn start_fade(&mut self, hwnd: HWND, opacity: f32) {
        let now = Instant::now();
        let from = self.fade.map_or(1.0, |t| t.value_at(now));
        let fade = self
            .motion
            .tween(from, opacity, motion::FAST, Curve::Linear, now);
        self.fade = Some(fade);
        window::set_timer(hwnd, TIMER_FADE, 16);
    }

    /// False when the GPU device is gone and a surface from the stack's current device (the
    /// app rebuilds it when a fence loses it too) cannot be drawn either.
    fn redraw(&mut self) -> bool {
        self.draw() || (self.panel.recreate(&self.stack).is_ok() && self.draw())
    }

    /// False on device loss.
    fn draw(&self) -> bool {
        let drawn = self.panel.draw(self.dpi, |s, _, _| {
            self.card
                .draw(s, &self.layout, &self.theme, self.hover, self.pressed)
        });
        !matches!(drawn, Ok(false))
    }

    fn hit(&self, lparam: isize) -> NoticeHit {
        let scale = self.dpi as f32 / 96.0;
        let x = msg::lo_i16(lparam) as f32 / scale;
        let y = msg::hi_i16(lparam) as f32 / scale;
        self.layout.hit(x, y)
    }
}

/// A visible notice; dropping it closes it.
pub struct Notice {
    window: Window,
    _target: DesktopWindowTarget,
    view: Rc<RefCell<View>>,
}

impl Notice {
    /// Shows `title`, `body` and an `action` button at the bottom-right of the primary
    /// monitor's work area, or at the nearest spot there that covers none of `avoid` (the
    /// fences: the first start's tips talk about them). `locale` is the text's language (fonts
    /// for Han, kana, Hangul). Clicks and the timeout arrive as [`Command::Notice`].
    pub fn show(
        ctx: &FenceContext,
        title: &str,
        body: &str,
        action: &str,
        locale: &str,
        avoid: &[RECT],
    ) -> Result<Self> {
        Self::show_with(ctx, title, body, action, None, locale, avoid)
    }

    /// [`Self::show`] with a plain `secondary` button left of the action (「不用」): it closes
    /// the notice like × and reports [`NoticeEvent::Declined`]; a click on the card's body
    /// then does nothing.
    pub fn show_with(
        ctx: &FenceContext,
        title: &str,
        body: &str,
        action: &str,
        secondary: Option<&str>,
        locale: &str,
        avoid: &[RECT],
    ) -> Result<Self> {
        let mut card = NoticeCard::new(title, body, action, locale)?;
        if let Some(text) = secondary {
            card = card.with_secondary(text);
        }
        Self::show_card(ctx, card, title, avoid)
    }

    /// [`Self::show`] for the first card of a row that [`Self::set_content`] goes through (the
    /// tips): the card is at least `min_height` DIPs tall, the height of the tallest one
    /// ([`card_height`]), so the later ones change its texts and not its size.
    pub fn show_reserving(
        ctx: &FenceContext,
        title: &str,
        body: &str,
        action: &str,
        min_height: f32,
        locale: &str,
        avoid: &[RECT],
    ) -> Result<Self> {
        let card = NoticeCard::new(title, body, action, locale)?.with_min_height(min_height);
        Self::show_card(ctx, card, title, avoid)
    }

    fn show_card(
        ctx: &FenceContext,
        card: NoticeCard,
        title: &str,
        avoid: &[RECT],
    ) -> Result<Self> {
        // A card still fading out goes now.
        LEAVING.with(|l| l.borrow_mut().clear());
        let monitor = monitors::enumerate()
            .into_iter()
            .find(|m| m.primary)
            .ok_or_else(windows_core::Error::empty)?;
        let dpi = monitor.dpi.max(96);
        let scale = dpi as f32 / 96.0;
        let layout = card.layout()?;
        let (w, h) = (
            (WIDTH * scale).round() as i32,
            (layout.height * scale).round() as i32,
        );
        let margin = (MARGIN * scale).round() as i32;
        let work = monitor.work_area;
        let corner = RECT {
            left: work.right - margin - w,
            top: work.bottom - margin - h,
            right: work.right - margin,
            bottom: work.bottom - margin,
        };
        let rect = if avoid
            .iter()
            .any(|a| crate::layout::clearance::overlaps(&corner, a, margin))
        {
            // A margin away from every fence where such a spot exists.
            let spot = crate::layout::clearance::nearest_free_apart(&corner, avoid, &work, margin);
            match spot {
                Some(r) => tracing::info!(
                    corner = ?(corner.left, corner.top),
                    to = ?(r.left, r.top),
                    avoided = avoid.len(),
                    "notice moved off the fences"
                ),
                None => tracing::info!(
                    corner = ?(corner.left, corner.top),
                    avoided = avoid.len(),
                    "notice stays in the corner over a fence: no free spot"
                ),
            }
            spot.unwrap_or(corner)
        } else {
            corner
        };

        NOTICE_CLASS.with(|c| -> Result<()> {
            if c.borrow().is_none() {
                *c.borrow_mut() = Some(WindowClass::register(CLASS, ClassOptions::default())?);
            }
            Ok(())
        })?;
        let view: Rc<RefCell<Option<Rc<RefCell<View>>>>> = Rc::new(RefCell::new(None));
        let handler_view = view.clone();
        let queue: CommandQueue = ctx.queue.clone();
        let window = NOTICE_CLASS.with(|c| {
            let c = c.borrow();
            let class = c.as_ref().ok_or_else(windows_core::Error::empty)?;
            WindowBuilder::new(class)
                .title(title)
                .style(style::POPUP)
                .ex_style(style::EX_TOPMOST | style::EX_TOOLWINDOW | style::EX_NOREDIRECTIONBITMAP)
                .bounds(rect.left, rect.top, w, h)
                .create(Box::new(move |hwnd, m, wparam, lparam| {
                    let view = handler_view.borrow().clone()?;
                    handle(hwnd, m, wparam, lparam, &view, &queue)
                }))
        })?;
        let hwnd = window.hwnd();
        let _ = dwm::set_corner_preference(hwnd, dwm::CornerPreference::Round);
        let _ = dwm::set_immersive_dark_mode(
            hwnd,
            matches!(ctx.theme.borrow().mode, pecofence_render::ThemeMode::Dark),
        );

        let target = ctx.stack.create_target(hwnd)?;
        let root = ctx.stack.compositor.create_container_visual();
        target.set_root(&root);
        let mut panel = Panel::new(&ctx.stack)?;
        panel.resize(w, h)?;
        root.children().insert_at_top(&panel.visual);
        let mut shadow = ShadowWindow::create(&ctx.shadow_class, ctx.shadow_style.get())?;
        shadow.update(rect, dpi);
        // Fades in like a fence: the first frame shows nothing.
        root.set_opacity(0.0);
        shadow.set_alpha(0);
        let shared = Rc::new(RefCell::new(View {
            panel,
            stack: ctx.stack.clone(),
            card,
            layout,
            theme: *ctx.theme.borrow(),
            dpi,
            hover: NoticeHit::None,
            pressed: NoticeHit::None,
            root,
            motion: ctx.motion.clone(),
            shadow,
            fade: Some(Tween::at(0.0, Instant::now())),
            closing: false,
            declined: false,
            left: false,
            held: false,
        }));
        shared.borrow_mut().redraw();
        *view.borrow_mut() = Some(shared.clone());

        window.show_no_activate();
        {
            let mut v = shared.borrow_mut();
            desktop::show_no_activate(v.shadow.hwnd());
            let _ = desktop::insert_after(v.shadow.hwnd(), hwnd);
            v.start_fade(hwnd, 1.0);
        }
        window.set_timer(TIMER_DISMISS, SHOW_FOR_MS);
        let notice = Self {
            window,
            _target: target,
            view: shared,
        };
        // A flyout already open over the corner (the taskbar's hidden icons) stays on top.
        notice.go_below(desktop::foreground_window());
        Ok(notice)
    }

    /// Puts other texts on the card where it stands (「下一条」: the next tip). The card stays
    /// on screen with no fade and keeps its size: texts that need less room leave it between
    /// the text and the button. Only texts that need more make it taller (its bottom edge
    /// keeps its place); [`Self::show_reserving`] gives the first card that room. The 30 s
    /// until it leaves by itself start again, and a card that was fading out comes back.
    pub fn set_content(&self, title: &str, body: &str, action: &str, locale: &str) -> Result<()> {
        let hwnd = self.window.hwnd();
        let old = window::window_rect(hwnd);
        let (rect, dpi) = {
            let mut v = self.view.borrow_mut();
            v.held = false;
            if std::mem::take(&mut v.closing) {
                tracing::debug!("notice was fading out: back for its new content");
                v.declined = false;
                v.start_fade(hwnd, 1.0);
            }
            let card =
                NoticeCard::new(title, body, action, locale)?.with_min_height(v.layout.height);
            let layout = card.layout()?;
            let scale = v.dpi as f32 / 96.0;
            let h = ((layout.height * scale).round() as i32).max(old.bottom - old.top);
            let rect = RECT {
                top: old.bottom - h,
                ..old
            };
            // The pointer rests where it clicked: the new card's part under it.
            let p = window::cursor_pos();
            let inside =
                (rect.left..rect.right).contains(&p.x) && (rect.top..rect.bottom).contains(&p.y);
            v.hover = if inside {
                layout.hit(
                    (p.x - rect.left) as f32 / scale,
                    (p.y - rect.top) as f32 / scale,
                )
            } else {
                NoticeHit::None
            };
            v.card = card;
            v.layout = layout;
            v.panel.resize(rect.right - rect.left, h)?;
            v.redraw();
            (rect, v.dpi)
        };
        tracing::debug!(
            from = ?(old.top, old.bottom),
            to = ?(rect.top, rect.bottom),
            grew = rect != old,
            "notice content swapped in place"
        );
        // The borrow is released first: the size change comes back into `handle`.
        if rect != old {
            self.window.set_bounds(
                rect.left,
                rect.top,
                rect.right - rect.left,
                rect.bottom - rect.top,
            )?;
            self.view.borrow_mut().shadow.update(rect, dpi);
        }
        window::set_title(hwnd, title);
        self.window.set_timer(TIMER_DISMISS, SHOW_FOR_MS);
        Ok(())
    }

    /// The card's window: its events name it ([`Command::Notice`]), and another card keeps
    /// clear of its place.
    pub fn hwnd(&self) -> HWND {
        self.window.hwnd()
    }

    /// The card was answered and the app keeps it while it acts on the answer, for a card that
    /// the answer brings up ([`Self::set_content`]): until then it takes no clicks and does
    /// not leave by itself.
    pub fn hold(&self) {
        window::kill_timer(self.window.hwnd(), TIMER_DISMISS);
        self.view.borrow_mut().held = true;
    }

    /// The card was answered (its button, or Ctrl+Z for what it offers to undo): it fades out
    /// as it does after × and the timeout, and reports nothing more. With Windows' animations
    /// off it goes at once.
    pub fn leave(self) {
        let hwnd = self.window.hwnd();
        window::kill_timer(hwnd, TIMER_DISMISS);
        let fades = {
            let mut v = self.view.borrow_mut();
            // A card that × or the timeout already faded out has nothing left to show.
            let fades = v.motion.enabled() && !v.closing;
            if fades {
                v.left = true;
                v.closing = true;
                v.start_fade(hwnd, 0.0);
            }
            fades
        };
        tracing::debug!(fades, "notice answered: it leaves");
        if fades {
            LEAVING.with(|l| l.borrow_mut().push(self));
        }
    }

    /// The front window changed: another program's topmost window over the card (the
    /// taskbar's hidden icons, Start, the notification centre) gets the card and its shadow
    /// just below it, so the card covers none of it. The card stays topmost, and clickable
    /// once that window closes.
    pub fn go_below(&self, foreground: HWND) {
        let hwnd = self.window.hwnd();
        let front = desktop::root_ancestor(foreground);
        if front.0.is_null() || front == hwnd || desktop::window_pid(front) == std::process::id() {
            return;
        }
        let class = desktop::class_name(front);
        let (card, other) = (window::window_rect(hwnd), window::window_rect(front));
        let over = card.left < other.right
            && other.left < card.right
            && card.top < other.bottom
            && other.top < card.bottom;
        let topmost = desktop::is_topmost(front);
        if !(topmost && over) {
            tracing::info!(%class, topmost, over, "notice stays above: the front window does not cover it");
            return;
        }
        let moved = desktop::insert_after(hwnd, front).is_ok();
        let _ = desktop::insert_after(self.view.borrow().shadow.hwnd(), hwnd);
        tracing::info!(%class, moved, "notice goes below the front window over it");
    }

    /// The app rebuilt the render stack (device loss): the card moves to a surface of the new
    /// device, or goes away when it cannot be drawn there either.
    pub fn redraw(&self) {
        let drawn = {
            let mut v = self.view.borrow_mut();
            v.closing || v.redraw()
        };
        if !drawn {
            fade_out(self.window.hwnd(), &self.view);
        }
    }
}

impl Drop for Notice {
    fn drop(&mut self) {
        self.view.borrow().shadow.hide();
        self.window.hide();
    }
}

/// × or the timeout: the app hears of it, and drops the notice, once it has faded out.
fn fade_out(hwnd: HWND, view: &Rc<RefCell<View>>) {
    let mut v = view.borrow_mut();
    if !std::mem::replace(&mut v.closing, true) {
        v.start_fade(hwnd, 0.0);
    }
}

fn handle(
    hwnd: HWND,
    m: u32,
    wparam: usize,
    lparam: isize,
    view: &Rc<RefCell<View>>,
    queue: &CommandQueue,
) -> Option<isize> {
    let set_hover = |hit: NoticeHit| {
        let mut v = view.borrow_mut();
        if v.hover != hit {
            v.hover = hit;
            // A failure means the device is gone and the app has not rebuilt the stack yet;
            // its rebuild redraws the card (`Notice::redraw`).
            v.redraw();
        }
    };
    // A card that is fading out, or kept for the next card's texts, takes no input.
    let inert = {
        let v = view.borrow();
        v.closing || v.held
    };
    if inert && m != msg::WM_TIMER {
        return None;
    }
    match m {
        msg::WM_MOUSEMOVE => {
            let hit = view.borrow().hit(lparam);
            set_hover(hit);
            window::track_mouse_leave(hwnd);
            Some(0)
        }
        msg::WM_MOUSELEAVE => {
            view.borrow_mut().pressed = NoticeHit::None;
            set_hover(NoticeHit::None);
            Some(0)
        }
        msg::WM_LBUTTONDOWN => {
            let mut v = view.borrow_mut();
            let hit = v.hit(lparam);
            if matches!(
                hit,
                NoticeHit::Button | NoticeHit::Secondary | NoticeHit::Close
            ) {
                v.pressed = hit;
                v.redraw();
            }
            Some(0)
        }
        msg::WM_SETCURSOR => {
            // The body of a card with two buttons does nothing: the arrow there.
            let v = view.borrow();
            if v.layout.has_secondary() && matches!(v.hover, NoticeHit::None | NoticeHit::Card) {
                return None;
            }
            window::set_standard_cursor(StandardCursor::Hand);
            Some(1)
        }
        msg::WM_LBUTTONUP => {
            let hit = view.borrow().hit(lparam);
            {
                let mut v = view.borrow_mut();
                if std::mem::replace(&mut v.pressed, NoticeHit::None) != NoticeHit::None {
                    v.redraw();
                }
            }
            let inert_body = hit == NoticeHit::Card && view.borrow().layout.has_secondary();
            if hit == NoticeHit::None || inert_body {
                return Some(0);
            }
            window::kill_timer(hwnd, TIMER_DISMISS);
            if hit == NoticeHit::Close {
                fade_out(hwnd, view);
            } else if hit == NoticeHit::Secondary {
                view.borrow_mut().declined = true;
                fade_out(hwnd, view);
            } else {
                // What the click opens should not wait for the exit fade.
                queue.push(Command::Notice(hwnd.0 as isize, NoticeEvent::Action));
            }
            Some(0)
        }
        msg::WM_TIMER if wparam == TIMER_DISMISS => {
            if view.borrow().hover != NoticeHit::None {
                window::set_timer(hwnd, TIMER_DISMISS, HOVER_RECHECK_MS);
            } else {
                window::kill_timer(hwnd, TIMER_DISMISS);
                fade_out(hwnd, view);
            }
            Some(0)
        }
        msg::WM_TIMER if wparam == TIMER_FADE => {
            let mut v = view.borrow_mut();
            let now = Instant::now();
            let Some(fade) = v.fade else {
                return Some(0);
            };
            let (opacity, done) = (fade.value_at(now), fade.is_done(now));
            v.root.set_opacity(opacity);
            v.shadow.set_alpha((opacity * 255.0).round() as u8);
            if done {
                v.fade = None;
                window::kill_timer(hwnd, TIMER_FADE);
                if v.closing && v.left {
                    // Nobody waits for this card: hidden here, dropped when the next one
                    // shows. Hiding sends messages back here: no borrow held.
                    v.shadow.hide();
                    drop(v);
                    desktop::hide_window(hwnd);
                } else if v.closing {
                    let event = if v.declined {
                        NoticeEvent::Declined
                    } else {
                        NoticeEvent::Dismissed
                    };
                    queue.push(Command::Notice(hwnd.0 as isize, event));
                }
            }
            Some(0)
        }
        _ => None,
    }
}
