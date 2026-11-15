//! 网页层 in the app: a web plugin fence's page (`crate::web_plugin`) and its optional main.js
//! beside it (the light layer), the page's `pf.*` calls, the events pages get, and a page that
//! crashed or cannot start (WebView2 missing): the fence shows why, with 重新加载.

use super::light::{note_stop, stopped_rows};
use super::*;
use crate::fence_window::{FenceWindow, ScriptView};
use crate::web_plugin::{Failure, WebFence};
use pecofence_core::i18n;
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::time::Instant;

const WEBVIEW2_DOWNLOAD: &str = "https://developer.microsoft.com/microsoft-edge/webview2/";

/// Why a fence's page does not run; it waits for 重新加载.
enum Stop {
    NoWebView2,
    Failed(Failure),
}

#[derive(Default)]
pub(super) struct Web {
    /// When each fence's page failed within the last hour.
    failures: HashMap<FenceId, Vec<Instant>>,
    stopped: HashMap<FenceId, Stop>,
}

impl App {
    /// `Command::StartWebFence`: the page of the web plugin the window of `host` shows, and the
    /// plugin's main.js when it has one (not started again while it runs).
    pub(super) fn start_web_fence(&mut self, host: FenceId) {
        let Some(shown) = self.fences.get(&host).map(|w| w.active_fence()) else {
            return;
        };
        let Some(plugin) = self.web_plugin_of(shown) else {
            return;
        };
        let w = self.fences.get(&host);
        if w.and_then(|w| w.web_plugin()).as_deref() == Some(&plugin.id)
            || self.web_pages.stopped.contains_key(&shown)
        {
            return;
        }
        // Made once, as for the settings window (it pumps messages until the browser is there).
        if !self.ensure_web_env() {
            tracing::warn!(fence = %shown, plugin = plugin.id, "web plugin: no WebView2, the fence says so");
            self.web_pages.stopped.insert(shown, Stop::NoWebView2);
            return self.push_tool_view(host);
        }
        let main = plugin.dir.join("main.js");
        if main.is_file() && !self.light_runs(shown) {
            self.start_light(shown, &plugin.id, &main);
        }
        if plugin.clipboard_read {
            self.ensure_clipboard_listener();
        }
        let init = self.plugin(&plugin.id);
        let init = init.map(|p| self.plugin_init(shown, &p.manifest));
        let (Some(w), Some(env), Some(init)) =
            (self.fences.get(&host), self.web_env.as_ref(), init)
        else {
            return;
        };
        let queue = self.queue.clone();
        let web = WebFence::start(
            &env.env,
            &self.ctx.stack,
            w.hwnd(),
            shown,
            plugin,
            queue,
            init,
        );
        w.set_web(Some(web));
    }

    /// `push_tool_view` of a web plugin fence: why its page stopped, or 正在运行… until the page
    /// is there.
    pub(super) fn push_web_view(&self, w: &FenceWindow, host: FenceId, fence: FenceId, id: &str) {
        if let Some(view) = self.web_stopped_view(fence) {
            w.set_web(None);
            w.set_script(Some(view));
        } else if w.web_plugin().as_deref() != Some(id) {
            w.set_script(Some(ScriptView {
                quiet: true,
                ..ScriptView::default()
            }));
            self.queue.push(Command::StartWebFence(host));
        }
    }

    /// A stopped page's fence: why, in red, then 重新加载 (and the download of WebView2 when it
    /// is missing).
    pub(super) fn web_stopped_view(&self, fence: FenceId) -> Option<ScriptView> {
        let (why, download) = match self.web_pages.stopped.get(&fence)? {
            Stop::NoWebView2 => (
                i18n::text("需要 WebView2 才能显示这个插件。").to_string(),
                true,
            ),
            Stop::Failed(f) => {
                let reason = match f {
                    Failure::BrowserExited => i18n::text("WebView2 的浏览器进程退出了"),
                    Failure::PageExited => i18n::text("网页的进程退出了"),
                    Failure::PageHung => i18n::text("网页没有响应"),
                };
                let text = i18n::format(
                    "插件的网页一小时里停了三次，最后一次是因为{0}。",
                    &[reason.to_string()],
                );
                (text, false)
            }
        };
        let mut buttons = Vec::new();
        if download {
            buttons.push(i18n::text("下载 WebView2"));
        }
        buttons.push(i18n::text("重新加载"));
        Some(ScriptView {
            rows: stopped_rows(&why, &buttons, false),
            ..ScriptView::default()
        })
    }

    /// Plugin fence `fence`'s version changed (an update, a return to the version before): a
    /// page it shows, or one stopped after its crashes, is made anew from the new folder.
    pub(super) fn restart_web_page(&mut self, fence: FenceId) {
        let was_stopped = self.web_pages.stopped.remove(&fence).is_some();
        let host = self.state.host_of(fence);
        let Some(w) = self.fences.get(&host) else {
            return;
        };
        let shows = w.web().is_some_and(|web| web.fence == fence);
        if shows {
            w.set_web(None);
        }
        if shows || was_stopped {
            tracing::info!(%fence, was_stopped, "web plugin: the page starts again for the new version");
            self.push_tool_view(host);
        }
    }

    /// A click in a stopped page's fence (true: it was one): 下载 WebView2 opens its page in the
    /// browser, 重新加载 starts the page again.
    pub(super) fn web_row_clicked(&mut self, fence: FenceId, row: usize, cell: usize) -> bool {
        let Some(stop) = self.web_pages.stopped.get(&fence) else {
            return false;
        };
        let download = matches!(stop, Stop::NoWebView2);
        if row != 1 {
            return true;
        }
        if download && cell == 0 {
            let owner = self.window_for(fence).map(|w| w.hwnd());
            let path = std::path::Path::new(WEBVIEW2_DOWNLOAD);
            let ok = shell::shell_execute(path, None, owner).is_ok();
            tracing::info!(%fence, ok, "web plugin: the WebView2 download page opened");
        } else {
            tracing::info!(%fence, "web plugin: 重新加载, the page starts again");
            self.web_pages.stopped.remove(&fence);
            self.push_tool_view(self.state.host_of(fence));
        }
        true
    }

    /// A page's process ended or hung: the page is made anew, unless it is its third time
    /// within an hour; then the fence says why and waits for 重新加载. When the browser process
    /// ended, the next page makes the environment anew.
    pub(super) fn web_page_failed(&mut self, page: Weak<WebFence>, failure: Failure) {
        let Some(web) = page.upgrade() else {
            return;
        };
        let fence = web.fence;
        let host = self.state.host_of(fence);
        let current = self.fences.get(&host).and_then(|w| w.web());
        if !current.is_some_and(|c| Rc::ptr_eq(&c, &web)) {
            // An older page of the fence (its failure came late).
            return;
        }
        drop(web);
        if failure == Failure::BrowserExited {
            self.web_env = None;
        }
        let (count, again) = note_stop(&mut self.web_pages.failures, fence);
        tracing::warn!(%fence, ?failure, count, again, "web plugin: page failed; made anew when again, else the fence shows why");
        if !again {
            self.web_pages.stopped.insert(fence, Stop::Failed(failure));
            // In the first minute after an update, back to the version before.
            self.plugin_stopped(fence);
        }
        if let Some(w) = self.fences.get(&host) {
            w.set_web(None);
        }
        self.push_tool_view(host);
    }

    /// A page's `pf.*` call: `post` goes to the plugin's main.js, `menu` (a right-click the
    /// page did not take) opens the fence's menu, the rest run as the light layer's calls do
    /// (`plugin_api`, the same permission checks). `pf.open` and the menu need a press in the
    /// page within the last second.
    pub(super) fn web_call(
        &mut self,
        page: Weak<WebFence>,
        id: serde_json::Value,
        call: String,
        args: serde_json::Value,
    ) {
        let Some(web) = page.upgrade() else {
            return;
        };
        let (fence, pressed) = (web.fence, web.pressed_lately());
        let plugin = web.plugin.id.clone();
        match call.as_str() {
            "post" => {
                let data = args.get(0).cloned().unwrap_or_default();
                tracing::debug!(plugin, %fence, "web plugin: the page posts to its main.js");
                self.plugin_event(fence, "message", data);
            }
            "menu" if pressed => {
                let p = window::cursor_pos();
                tracing::debug!(plugin, %fence, "web plugin: right-click on the page, the fence menu");
                self.show_fence_menu(fence, p.x, p.y);
            }
            "menu" | "open" if !pressed => {
                tracing::info!(plugin, %fence, call, "web plugin: refused, no click in the fence just before");
                web.reply(
                    &id,
                    Err(format!(
                        "pf.{call} works only right after a click in the fence"
                    )),
                );
            }
            _ => {
                tracing::debug!(plugin, %fence, call, "web plugin: pf call");
                drop(web);
                self.plugin_api(fence, &call, args, move |result| {
                    if let Some(web) = page.upgrade() {
                        web.reply(&id, result);
                    }
                });
            }
        }
    }

    /// Event `name` for fence `fence`'s page, when it shows one.
    pub(super) fn web_event(&self, fence: FenceId, name: &str, data: serde_json::Value) {
        let web = self.window_for(fence).and_then(|w| w.web());
        if let Some(web) = web.filter(|w| w.fence == fence) {
            web.event(name, &data);
        }
    }

    /// Event `name` for every page (`clipboard` only for the plugins that may read it).
    pub(super) fn web_events(&self, name: &str, data: serde_json::Value) {
        for web in self.fences.values().filter_map(|w| w.web()) {
            if name != "clipboard" || web.plugin.clipboard_read {
                web.event(name, &data);
            }
        }
    }
}
