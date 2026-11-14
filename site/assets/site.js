(function () {
  "use strict";

  // Keep the reader's place when switching language.
  var select = document.getElementById("language-select");
  if (select) {
    select.addEventListener("change", function () {
      var target = select.options[select.selectedIndex].getAttribute("data-href");
      if (target) window.location.href = target + window.location.hash;
    });
  }

  // Each feature shows the manual's animation of it (manual/?only=<lesson>&embed) in place of
  // the recording, loaded the first time its tab opens; the recording stays without JS.
  function showDemo(panel) {
    var frame = panel.querySelector(".feature-demo");
    if (!frame || frame.hidden === false) return;
    var media = frame.parentNode;
    media.querySelector("video").controls = false;
    frame.addEventListener("load", function () { media.classList.add("has-demo"); });
    frame.hidden = false;
    frame.src = frame.getAttribute("data-src");
  }

  // Without JS every feature remains visible; enhance to a keyboard-friendly gallery.
  var picker = document.querySelector(".feature-picker");
  if (picker) {
    var tabs = Array.prototype.slice.call(picker.querySelectorAll('[role="tab"]'));
    function activateTab(tab, focus) {
      tabs.forEach(function (item) {
        var active = item === tab;
        var panel = document.getElementById(item.getAttribute("aria-controls"));
        item.setAttribute("aria-selected", String(active));
        item.tabIndex = active ? 0 : -1;
        panel.hidden = !active;
        panel.setAttribute("role", "tabpanel");
        panel.setAttribute("aria-labelledby", item.id);
        panel.tabIndex = 0;
        if (active) showDemo(panel);
        else panel.querySelector("video").pause();
      });
      if (focus) {
        tab.focus({ preventScroll: true });
        tab.scrollIntoView({ block: "nearest", inline: "nearest" });
      }
    }
    picker.hidden = false;
    picker.closest(".features-section").classList.add("features-enhanced");
    tabs.forEach(function (tab, index) {
      tab.addEventListener("click", function () { activateTab(tab, false); });
      tab.addEventListener("keydown", function (event) {
        var next;
        if (event.key === "ArrowRight") next = (index + 1) % tabs.length;
        if (event.key === "ArrowLeft") next = (index + tabs.length - 1) % tabs.length;
        if (event.key === "Home") next = 0;
        if (event.key === "End") next = tabs.length - 1;
        if (next !== undefined) {
          event.preventDefault();
          activateTab(tabs[next], true);
        }
      });
    });
    activateTab(tabs[0], false);
  }

  // The tools block under the gallery shows the manual's tools lesson the same way.
  var tools = document.querySelector(".tools");
  if (tools) showDemo(tools);

  // The trailer shows its poster and a large play control. It never starts on its own;
  // a click plays it inline with sound and hands over to the native controls.
  // Without JavaScript the native controls are there from the start.
  var trailer = document.querySelector(".trailer");
  if (trailer) {
    var film = trailer.querySelector("video");
    var start = trailer.querySelector(".trailer-play");
    film.controls = false;
    start.hidden = false;
    start.addEventListener("click", function () {
      start.hidden = true;
      film.controls = true;
      film.muted = false;
      film.focus({ preventScroll: true });
      var playing = film.play();
      if (playing && playing.catch) playing.catch(function () {});
    });
  }

  document.querySelectorAll(".copy-command").forEach(function (copy) {
    if (!navigator.clipboard || !window.isSecureContext) return;
    copy.hidden = false;
    var copyReset;
    copy.addEventListener("click", function () {
      navigator.clipboard.writeText(copy.getAttribute("data-command")).then(function () {
        window.clearTimeout(copyReset);
        copy.querySelector("span").textContent = copy.getAttribute("data-copied");
        copy.querySelector("use").setAttribute("href", "#i-check");
        copyReset = window.setTimeout(function () {
          copy.querySelector("span").textContent = copy.getAttribute("data-copy");
          copy.querySelector("use").setAttribute("href", "#i-copy");
        }, 2200);
      }).catch(function () {
        // Keep the corresponding prompt or command selectable when copying is blocked.
        var container = copy.closest("[data-copy-container]");
        var content = container && (container.querySelector("code") || container.querySelector("[data-copy-content]"));
        if (content) {
          var range = document.createRange();
          range.selectNodeContents(content);
          var selection = window.getSelection();
          selection.removeAllRanges();
          selection.addRange(range);
        }
      });
    });
  });

  // Pause background playback when the page is no longer visible.
  document.addEventListener("visibilitychange", function () {
    if (document.hidden) {
      document.querySelectorAll("video").forEach(function (video) { video.pause(); });
    }
  });
})();
