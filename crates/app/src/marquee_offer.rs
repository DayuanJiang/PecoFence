//! After a rectangle on the empty desktop: two small buttons at its bottom-right corner,
//! 「在此显示文件夹…」 and 「＋ 新建栅栏」 (`pecofence_render::marquee_offer`). They take neither the
//! foreground nor the keyboard, so Delete, Esc and a new drag keep working on the desktop;
//! they fade out after [`SHOW_FOR_MS`] unless the pointer rests on them, and go at once on Esc
//! or a mouse press anywhere else. A click arrives as [`Command::MarqueeOffer`].

use crate::commands::{Command, CommandQueue};
use crate::fence_window::FenceContext;
use pecofence_platform::window::{
    self, ClassOptions, StandardCursor, Window, WindowBuilder, WindowClass, style,
};
use pecofence_platform::{HWND, RECT, desktop, monitors, msg};
use pecofence_render::marquee_offer::{OfferButtons, OfferHit, OfferLayout};
use pecofence_render::motion::{self, Curve, Motion, Tween};
use pecofence_render::{ContainerVisual, DesktopWindowTarget, Panel, RenderStack, Theme};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use windows_core::Result;

pub const OFFER_CLASS: &str = "PecoFence.MarqueeOffer";
/// How long the buttons stay when the pointer leaves them alone (the decision: about 3 s).
const SHOW_FOR_MS: u128 = 3_000;
/// Esc and presses elsewhere are polled at this interval while the buttons show.
const TIMER_POLL: usize = 1;
const POLL_MS: u32 = 30;
/// Steps the fade in and out.
const TIMER_FADE: usize = 2;
/// Space between the rectangle's bottom edge and the buttons, in DIPs.
const GAP: f32 = 8.0;
const VK_LBUTTON: u32 = 0x01;
const VK_RBUTTON: u32 = 0x02;
const VK_MBUTTON: u32 = 0x04;
const VK_ESCAPE: u32 = 0x1B;
const WATCHED: [u32; 4] = [VK_ESCAPE, VK_LBUTTON, VK_RBUTTON, VK_MBUTTON];

/// What became of the buttons.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OfferEvent {
    NewFence,
    ShowFolder,
    /// Faded out after the timeout, or Esc / a press elsewhere (`reason` for the log).
    Dismissed(&'static str),
    /// The buttons are fading out by themselves: how opaque they are now (255: fully), so the
    /// rectangle they belong to fades with them.
    Fading(u8),
}

struct View {
    panel: Panel,
    stack: Rc<RenderStack>,
    buttons: OfferButtons,
    layout: OfferLayout,
    theme: Theme,
    dpi: u32,
    hover: OfferHit,
    pressed: OfferHit,
    root: ContainerVisual,
    motion: Rc<Motion>,
    fade: Option<Tween>,
    shown_at: Instant,
    /// Fading out: the buttons no longer react.
    closing: bool,
    /// Sent to the app when the fade-out lands.
    outcome: Option<OfferEvent>,
}

impl View {
    fn draw(&mut self) -> bool {
        let drawn = |v: &Self| {
            v.panel.draw(v.dpi, |s, _, _| {
                v.buttons.draw(s, &v.layout, &v.theme, v.hover, v.pressed)
            })
        };
        let ok = !matches!(drawn(self), Ok(false));
        ok || (self.panel.recreate(&self.stack).is_ok() && !matches!(drawn(self), Ok(false)))
    }

    fn hit(&self, lparam: isize) -> OfferHit {
        let scale = self.dpi as f32 / 96.0;
        let x = msg::lo_i16(lparam) as f32 / scale;
        let y = msg::hi_i16(lparam) as f32 / scale;
        self.layout.hit(x, y)
    }

    fn start_fade(&mut self, hwnd: HWND, to: f32) {
        let now = Instant::now();
        let from = self.fade.map_or(1.0, |t| t.value_at(now));
        let dur = if to > 0.5 {
            motion::FASTER
        } else {
            motion::FAST
        };
        self.fade = Some(self.motion.tween(from, to, dur, Curve::Linear, now));
        window::set_timer(hwnd, TIMER_FADE, 16);
    }
}

/// The buttons on screen; dropping them closes them.
pub struct MarqueeOffer {
    window: Window,
    _target: DesktopWindowTarget,
    _class: WindowClass,
}

impl MarqueeOffer {
    /// Shows the buttons under `rect`'s bottom-right corner (screen px), right edges lined up;
    /// above the corner, inside the rectangle, when there is no room under it.
    pub fn show(ctx: &FenceContext, rect: RECT) -> Result<Self> {
        let monitor = monitors::query(monitors::monitor_from_point(
            rect.right - 1,
            rect.bottom - 1,
        ))
        .ok_or_else(windows_core::Error::empty)?;
        let dpi = monitor.dpi.max(96);
        let scale = dpi as f32 / 96.0;
        let t = pecofence_core::i18n::text;
        let buttons =
            OfferButtons::new(t("新建栅栏"), t("在此显示文件夹…"), crate::notice::locale())?;
        let layout = buttons.layout()?;
        let (w, h) = (
            (layout.width * scale).ceil() as i32,
            (layout.height * scale).ceil() as i32,
        );
        let gap = (GAP * scale).round() as i32;
        let work = monitor.work_area;
        let mut y = rect.bottom + gap;
        if y + h > work.bottom {
            y = rect.bottom - gap - h;
        }
        let x = (rect.right - w).clamp(work.left, (work.right - w).max(work.left));
        let y = y.clamp(work.top, (work.bottom - h).max(work.top));

        let class = WindowClass::register(OFFER_CLASS, ClassOptions::default())?;
        let view: Rc<RefCell<Option<Rc<RefCell<View>>>>> = Rc::new(RefCell::new(None));
        let handler_view = view.clone();
        let queue: CommandQueue = ctx.queue.clone();
        let window = WindowBuilder::new(&class)
            // Both labels, for screen readers and test scripts (the buttons are drawn).
            .title(&format!("{}|{}", t("在此显示文件夹…"), t("新建栅栏")))
            .style(style::POPUP)
            .ex_style(
                style::EX_TOPMOST
                    | style::EX_TOOLWINDOW
                    | style::EX_NOACTIVATE
                    | style::EX_NOREDIRECTIONBITMAP,
            )
            .bounds(x, y, w, h)
            .create(Box::new(move |hwnd, m, wparam, lparam| {
                let view = handler_view.borrow().clone()?;
                handle(hwnd, m, wparam, lparam, &view, &queue)
            }))?;
        let hwnd = window.hwnd();
        let target = ctx.stack.create_target(hwnd)?;
        let root = ctx.stack.compositor.create_container_visual();
        target.set_root(&root);
        let mut panel = Panel::new(&ctx.stack)?;
        panel.resize(w, h)?;
        root.children().insert_at_top(&panel.visual);
        root.set_opacity(0.0);
        let shared = Rc::new(RefCell::new(View {
            panel,
            stack: ctx.stack.clone(),
            buttons,
            layout,
            theme: *ctx.theme.borrow(),
            dpi,
            hover: OfferHit::None,
            pressed: OfferHit::None,
            root,
            motion: ctx.motion.clone(),
            fade: Some(Tween::at(0.0, Instant::now())),
            shown_at: Instant::now(),
            closing: false,
            outcome: None,
        }));
        if !shared.borrow_mut().draw() {
            return Err(windows_core::Error::empty());
        }
        *view.borrow_mut() = Some(shared.clone());
        // Forget presses from before (the marquee's own button press): only new ones count.
        for vk in WATCHED {
            window::key_pressed_async(vk);
        }
        window.show_no_activate();
        shared.borrow_mut().start_fade(hwnd, 1.0);
        window.set_timer(TIMER_POLL, POLL_MS);
        tracing::info!(?rect, x, y, w, h, dpi, "marquee offer shown");
        Ok(Self {
            window,
            _target: target,
            _class: class,
        })
    }
}

impl Drop for MarqueeOffer {
    fn drop(&mut self) {
        self.window.hide();
    }
}

/// Fades out; the app hears `outcome` once the fade lands (a click is sent at once instead,
/// so what it opens does not wait).
fn close(hwnd: HWND, view: &Rc<RefCell<View>>, outcome: Option<OfferEvent>) {
    let mut v = view.borrow_mut();
    if !std::mem::replace(&mut v.closing, true) {
        window::kill_timer(hwnd, TIMER_POLL);
        v.outcome = outcome;
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
    let closing = view.borrow().closing;
    match m {
        msg::WM_MOUSEACTIVATE => Some(msg::MA_NOACTIVATE),
        msg::WM_SETCURSOR => {
            window::set_standard_cursor(StandardCursor::Hand);
            Some(1)
        }
        msg::WM_MOUSEMOVE if !closing => {
            let hit = view.borrow().hit(lparam);
            let mut v = view.borrow_mut();
            if v.hover != hit {
                v.hover = hit;
                v.draw();
            }
            drop(v);
            window::track_mouse_leave(hwnd);
            Some(0)
        }
        msg::WM_MOUSELEAVE => {
            let mut v = view.borrow_mut();
            if v.hover != OfferHit::None {
                v.hover = OfferHit::None;
                v.draw();
            }
            Some(0)
        }
        msg::WM_LBUTTONDOWN if !closing => {
            let mut v = view.borrow_mut();
            v.pressed = v.hit(lparam);
            v.draw();
            Some(0)
        }
        msg::WM_LBUTTONUP if !closing => {
            let (hit, pressed) = {
                let mut v = view.borrow_mut();
                let hit = v.hit(lparam);
                (hit, std::mem::take(&mut v.pressed))
            };
            let event = match (hit, pressed) {
                (OfferHit::NewFence, OfferHit::NewFence) => Some(OfferEvent::NewFence),
                (OfferHit::Folder, OfferHit::Folder) => Some(OfferEvent::ShowFolder),
                _ => None,
            };
            if let Some(event) = event {
                // The buttons fade out (167 ms) while what the click made fades in; they
                // were hidden in one frame here. The app keeps them alive meanwhile.
                close(hwnd, view, None);
                queue.push(Command::MarqueeOffer(event));
            } else {
                view.borrow_mut().draw();
            }
            Some(0)
        }
        msg::WM_TIMER if wparam == TIMER_POLL => {
            let (over, age) = {
                let v = view.borrow();
                let p = window::cursor_pos();
                let r = window::window_rect(hwnd);
                let over = p.x >= r.left && p.x < r.right && p.y >= r.top && p.y < r.bottom;
                (over, v.shown_at.elapsed().as_millis())
            };
            let esc = window::key_pressed_async(VK_ESCAPE);
            let pressed = [VK_LBUTTON, VK_RBUTTON, VK_MBUTTON]
                .into_iter()
                .fold(false, |any, vk| window::key_pressed_async(vk) | any);
            if esc {
                close(hwnd, view, Some(OfferEvent::Dismissed("Esc")));
            } else if pressed && !over {
                close(hwnd, view, Some(OfferEvent::Dismissed("press elsewhere")));
            } else if age >= SHOW_FOR_MS && !over {
                close(hwnd, view, Some(OfferEvent::Dismissed("timeout")));
            }
            Some(0)
        }
        msg::WM_TIMER if wparam == TIMER_FADE => {
            let mut v = view.borrow_mut();
            let now = Instant::now();
            let Some(fade) = v.fade else {
                return Some(0);
            };
            let opacity = fade.value_at(now);
            v.root.set_opacity(opacity);
            // Leaving by themselves (not after a click, which hid them at once).
            if v.closing && v.outcome.is_some() {
                let step = OfferEvent::Fading((opacity.clamp(0.0, 1.0) * 255.0).round() as u8);
                queue.push(Command::MarqueeOffer(step));
            }
            if fade.is_done(now) {
                v.fade = None;
                window::kill_timer(hwnd, TIMER_FADE);
                let gone = v.closing.then(|| v.outcome.take());
                // Hiding sends messages back here: no borrow held.
                drop(v);
                if let Some(outcome) = gone {
                    desktop::hide_window(hwnd);
                    if let Some(outcome) = outcome {
                        queue.push(Command::MarqueeOffer(outcome));
                    }
                }
            }
            Some(0)
        }
        msg::WM_CLOSE => Some(0),
        _ => None,
    }
}
