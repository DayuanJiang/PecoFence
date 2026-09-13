(function () {
  "use strict";

  // Language picker in the taskbar tray: go to the chosen language's page.
  var select = document.getElementById("language-select");
  if (select) {
    select.addEventListener("change", function () {
      var target = select.options[select.selectedIndex].getAttribute("data-href");
      if (target) window.location.href = target + window.location.hash;
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
    function reflect() {
      fence.classList.toggle("is-playing", !video.paused);
      toggle.setAttribute("aria-label", video.paused ? toggle.getAttribute("data-play") : toggle.getAttribute("data-pause"));
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
          if (video.getAttribute("data-user-paused") !== "1") video.play().catch(function () {});
        } else if (!video.paused) {
          video.pause();
        }
      });
    }, { threshold: [0, 0.5] });
    videos.forEach(function (video) { observer.observe(video); });
  }
})();
