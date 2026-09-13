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

  // A small, optional demonstration of hiding desktop fences.
  var preview = document.querySelector(".desktop-preview");
  var demoToggle = document.querySelector(".demo-toggle");
  if (preview && demoToggle) {
    var desk = preview.querySelector(".desk");
    var deskDescription = desk.getAttribute("aria-label");
    demoToggle.hidden = false;
    demoToggle.addEventListener("click", function () {
      var clear = preview.classList.toggle("is-clear");
      demoToggle.setAttribute("aria-pressed", String(clear));
      demoToggle.querySelector("span").textContent = demoToggle.getAttribute(clear ? "data-show" : "data-hide");
      desk.setAttribute("aria-label", clear ? preview.querySelector(".desktop-message").textContent : deskDescription);
    });
  }

  var copy = document.querySelector(".copy-command");
  if (copy && navigator.clipboard && window.isSecureContext) {
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
        // The command remains selectable if browser permissions block copying.
        var code = document.querySelector(".command-install code");
        if (code) {
          var range = document.createRange();
          range.selectNodeContents(code);
          var selection = window.getSelection();
          selection.removeAllRanges();
          selection.addRange(range);
        }
      });
    });
  }

  // Feature clips play while at least half visible and pause off-screen. They never
  // start on their own when the user prefers reduced motion. The button on each
  // fence toggles playback and remembers a deliberate pause.
  var reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  var videos = Array.prototype.slice.call(document.querySelectorAll(".clip video"));
  videos.forEach(function (video) {
    var fence = video.closest(".clip");
    var toggle = fence.querySelector(".clip-toggle");
    toggle.hidden = false;
    function reflect() {
      fence.classList.toggle("is-playing", !video.paused);
      var action = toggle.getAttribute(video.paused ? "data-play" : "data-pause");
      toggle.setAttribute("aria-label", action + ": " + toggle.getAttribute("data-title"));
    }
    toggle.addEventListener("click", function () {
      if (video.paused) {
        video.setAttribute("data-user-paused", "0");
        video.play().catch(function () {});
      } else {
        video.setAttribute("data-user-paused", "1");
        video.pause();
      }
    });
    video.addEventListener("play", reflect);
    video.addEventListener("pause", reflect);
    reflect();
  });
  if (!reduce && "IntersectionObserver" in window) {
    var observer = new IntersectionObserver(function (entries) {
      entries.forEach(function (entry) {
        var video = entry.target;
        if (entry.isIntersecting && entry.intersectionRatio >= 0.5) {
          if (!document.hidden && video.getAttribute("data-user-paused") !== "1") video.play().catch(function () {});
        } else if (!video.paused) {
          video.pause();
        }
      });
    }, { threshold: [0, 0.5] });
    videos.forEach(function (video) { observer.observe(video); });
  }
  // Pause background playback when the page is no longer visible.
  document.addEventListener("visibilitychange", function () {
    if (document.hidden) {
      document.querySelectorAll("video").forEach(function (video) { video.pause(); });
    } else if (observer) {
      videos.forEach(function (video) {
        observer.unobserve(video);
        observer.observe(video);
      });
    }
  });
})();
