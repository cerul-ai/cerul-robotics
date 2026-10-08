// Shared behaviour for the Cerul Robotics pages: episode clocks synced to
// video, scroll reveals, count-ups, cursor spotlight and docs copy buttons.
// Decorative motion follows prefers-reduced-motion. The muted product demo
// video plays regardless, because it is the content, but it stops with the
// on-page pause control, while offscreen, and while the tab is hidden.
(function () {
  "use strict";

  var root = document.documentElement;
  var reduce = window.matchMedia("(prefers-reduced-motion: reduce)");
  var STORE = "cerul-robotics-motion";

  function stored() {
    try { return window.localStorage.getItem(STORE); } catch (e) { return null; }
  }
  function store(value) {
    try { window.localStorage.setItem(STORE, value); } catch (e) { /* storage unavailable */ }
  }

  var userOff = stored() === "off";
  function motionOn() { return !reduce.matches && !userOff; }
  function videoOn() { return !userOff; }

  // Labels generated for episode 6774a70f/0 (see media/summary.md).
  var EPISODE = {
    duration: 7,
    fps: 30,
    subtask: [
      [0, 2.2, "Adjusting and separating warp threads across the reed"],
      [2.2, 4.8, "Repositioning left hand and adjusting warp threads across the reed"],
      [4.8, 5.4, "Retracting the left hand from the loom towards the lap"],
      [5.4, 7, "Reaching forward with the left hand and adjusting warp threads"]
    ],
    event: [
      [0, 1.5, "pull", "thread → loom"],
      [3.3, 4.5, "pull", "thread → loom"],
      [5.9, 7, "pull", "thread → loom"]
    ]
  };

  function find(list, t) {
    for (var i = 0; i < list.length; i++) {
      if (t >= list[i][0] && t < list[i][1]) return list[i];
    }
    return null;
  }
  function fmt(n) { return n.toFixed(3); }

  function swapText(el, text) {
    if (!el || el.textContent === text) return;
    if (!motionOn()) { el.textContent = text; return; }
    el.classList.add("out");
    window.setTimeout(function () {
      el.textContent = text;
      el.classList.remove("out");
    }, 180);
  }

  // One clock per [data-episode] block.
  var episodes = [];
  Array.prototype.forEach.call(document.querySelectorAll("[data-episode]"), function (block) {
    var video = block.querySelector("video[data-episode-video]") ||
      (block.dataset.videoFrom ? document.querySelector(block.dataset.videoFrom) : null);
    episodes.push({
      block: block,
      video: video,
      still: parseFloat(block.dataset.still || "3.9"),
      visible: false,
      started: 0,
      offset: 0,
      playhead: block.querySelector("[data-playhead]"),
      time: block.querySelector("[data-time]"),
      bars: block.querySelectorAll("[data-s][data-e]"),
      frames: block.querySelectorAll("[data-t]"),
      subtaskTitle: block.querySelector("[data-callout-subtask-time]"),
      subtaskText: block.querySelector("[data-callout-subtask-text]"),
      eventCard: block.querySelector("[data-callout-event]"),
      eventText: block.querySelector("[data-callout-event-text]"),
      handText: block.querySelector("[data-callout-hands]"),
      lastSubtask: null,
      lastEvent: undefined
    });
  });

  function clockTime(ep, now) {
    if (ep.video && ep.video.readyState >= 1 && !isNaN(ep.video.duration)) {
      return ep.video.currentTime % EPISODE.duration;
    }
    if (!motionOn() || !ep.visible) return ep.offset;
    return (ep.offset + (now - ep.started) / 1000) % EPISODE.duration;
  }

  function render(ep, t) {
    if (ep.playhead) ep.playhead.style.left = (t / EPISODE.duration * 100) + "%";
    if (ep.time) ep.time.textContent = fmt(t) + " s";
    Array.prototype.forEach.call(ep.bars, function (bar) {
      var on = t >= parseFloat(bar.dataset.s) && t < parseFloat(bar.dataset.e);
      bar.classList.toggle("on", on);
    });
    if (ep.frames.length) {
      var best = null;
      Array.prototype.forEach.call(ep.frames, function (frame) {
        if (parseFloat(frame.dataset.t) <= t + 0.05) best = frame;
      });
      Array.prototype.forEach.call(ep.frames, function (frame) {
        frame.classList.toggle("on", frame === best);
      });
    }
    var sub = find(EPISODE.subtask, t);
    if (sub && sub !== ep.lastSubtask) {
      ep.lastSubtask = sub;
      swapText(ep.subtaskTitle, "subtask · " + fmt(sub[0]) + "–" + fmt(sub[1]) + " s");
      swapText(ep.subtaskText, sub[2]);
    }
    var ev = find(EPISODE.event, t);
    if (ep.eventCard && ev !== ep.lastEvent) {
      ep.lastEvent = ev;
      ep.eventCard.classList.toggle("idle", !ev);
      swapText(ep.eventText, ev ? ev[2] + " · " + ev[3] : "waiting for contact");
    }
    if (ep.handText) {
      var frame = Math.min(Math.floor(t * EPISODE.fps) + 1, EPISODE.duration * EPISODE.fps);
      ep.handText.textContent = "frame " + String(frame).padStart(3, "0") + " / 210";
    }
  }

  function tick(now) {
    episodes.forEach(function (ep) {
      if (ep.visible || !ep.rendered) {
        render(ep, clockTime(ep, now));
        ep.rendered = true;
      }
    });
    window.requestAnimationFrame(tick);
  }

  // Videos marked data-autoplay play only while motion is on and they are visible.
  var autoVideos = Array.prototype.slice.call(document.querySelectorAll("video[data-autoplay]"));
  var videoVisible = new WeakMap();
  // Assume visible until the observer reports otherwise, so playback does not
  // depend on the observer's first callback.
  autoVideos.forEach(function (v) { videoVisible.set(v, true); });
  function syncVideo(video) {
    var want = videoOn() && videoVisible.get(video) && !document.hidden;
    if (video.dataset.userPaused === "true") want = false;
    if (video.dataset.userPlayed === "true" && !document.hidden) want = true;
    if (want && video.paused) {
      var p = video.play();
      if (p && p.catch) p.catch(function () { /* autoplay refused; poster stays */ });
    } else if (!want && !video.paused) {
      video.pause();
    }
    syncPlayButtons();
  }
  function syncAll() {
    autoVideos.forEach(syncVideo);
    root.classList.toggle("motion-off", userOff);
    episodes.forEach(function (ep) {
      if (!ep.video) {
        ep.offset = clockTime(ep, performance.now());
        ep.started = performance.now();
      }
    });
    Array.prototype.forEach.call(document.querySelectorAll("[data-motion-toggle]"), function (btn) {
      btn.setAttribute("aria-pressed", String(userOff));
      var label = btn.querySelector("[data-motion-label]");
      if (label) label.textContent = userOff ? "Play motion" : "Pause motion";
      var icon = btn.querySelector("[data-motion-icon]");
      if (icon) icon.setAttribute("d", userOff ? "M5 3.5v9l7-4.5z" : "M5 3.5h2v9H5zM9 3.5h2v9H9z");
    });
  }

  // Showcase play buttons control their own video explicitly.
  var playButtons = Array.prototype.slice.call(document.querySelectorAll("[data-play-for]"));
  function syncPlayButtons() {
    playButtons.forEach(function (btn) {
      var video = document.querySelector(btn.dataset.playFor);
      if (!video) return;
      var playing = !video.paused;
      btn.setAttribute("aria-label", playing ? "Pause episode video" : "Play episode video");
      btn.classList.toggle("is-playing", playing);
      var icon = btn.querySelector("path");
      if (icon) icon.setAttribute("d", playing ? "M7 5h3.5v14H7zM13.5 5H17v14h-3.5z" : "M8 5.5v13l10.5-6.5z");
    });
  }
  playButtons.forEach(function (btn) {
    btn.addEventListener("click", function () {
      var video = document.querySelector(btn.dataset.playFor);
      if (!video) return;
      if (video.paused) {
        video.dataset.userPlayed = "true";
        video.dataset.userPaused = "false";
        var p = video.play();
        if (p && p.catch) p.catch(function () {});
      } else {
        video.dataset.userPaused = "true";
        video.dataset.userPlayed = "false";
        video.pause();
      }
      syncPlayButtons();
    });
  });
  autoVideos.forEach(function (video) {
    video.addEventListener("play", syncPlayButtons);
    video.addEventListener("pause", syncPlayButtons);
  });

  Array.prototype.forEach.call(document.querySelectorAll("[data-motion-toggle]"), function (btn) {
    btn.addEventListener("click", function () {
      userOff = !userOff;
      store(userOff ? "off" : "on");
      autoVideos.forEach(function (v) { v.dataset.userPaused = "false"; v.dataset.userPlayed = "false"; });
      syncAll();
    });
  });

  if ("IntersectionObserver" in window) {
    var seen = new IntersectionObserver(function (entries) {
      entries.forEach(function (entry) {
        var el = entry.target;
        if (el.tagName === "VIDEO") {
          videoVisible.set(el, entry.isIntersecting);
          syncVideo(el);
        }
        episodes.forEach(function (ep) {
          if (ep.block === el) {
            if (entry.isIntersecting && !ep.visible) ep.started = performance.now();
            if (!entry.isIntersecting && ep.visible) ep.offset = clockTime(ep, performance.now());
            ep.visible = entry.isIntersecting;
          }
        });
      });
    }, { threshold: 0.15 });
    autoVideos.forEach(function (v) { seen.observe(v); });
    episodes.forEach(function (ep) { seen.observe(ep.block); });

    var reveal = new IntersectionObserver(function (entries) {
      entries.forEach(function (entry) {
        if (!entry.isIntersecting) return;
        entry.target.classList.add("in");
        reveal.unobserve(entry.target);
        Array.prototype.forEach.call(entry.target.querySelectorAll("[data-count]"), countUp);
        if (entry.target.hasAttribute("data-count")) countUp(entry.target);
      });
    }, { threshold: 0.2, rootMargin: "0px 0px -8% 0px" });
    Array.prototype.forEach.call(document.querySelectorAll(".reveal, [data-count]"), function (el) {
      reveal.observe(el);
    });
  } else {
    autoVideos.forEach(function (v) { videoVisible.set(v, true); });
    episodes.forEach(function (ep) { ep.visible = true; });
    Array.prototype.forEach.call(document.querySelectorAll(".reveal"), function (el) { el.classList.add("in"); });
  }

  function countUp(el) {
    if (el.dataset.counted) return;
    el.dataset.counted = "true";
    var target = parseFloat(el.dataset.count);
    var suffix = el.dataset.suffix || "";
    if (!motionOn() || target === 0) { el.textContent = target + suffix; return; }
    var start = performance.now();
    var span = 1300;
    (function step(now) {
      var p = Math.min((now - start) / span, 1);
      var eased = 1 - Math.pow(1 - p, 3);
      el.textContent = Math.round(target * eased) + suffix;
      if (p < 1) window.requestAnimationFrame(step);
    })(start);
  }

  // Cursor spotlight.
  Array.prototype.forEach.call(document.querySelectorAll(".spot"), function (card) {
    card.addEventListener("pointermove", function (event) {
      var r = card.getBoundingClientRect();
      card.style.setProperty("--mx", (event.clientX - r.left) + "px");
      card.style.setProperty("--my", (event.clientY - r.top) + "px");
    });
  });

  // Hover previews: play on pointer enter, rest on the poster otherwise.
  Array.prototype.forEach.call(document.querySelectorAll("[data-hover-video]"), function (card) {
    var video = card.querySelector("video");
    if (!video) return;
    card.addEventListener("pointerenter", function () {
      if (!videoOn()) return;
      var p = video.play();
      if (p && p.catch) p.catch(function () {});
    });
    card.addEventListener("pointerleave", function () { video.pause(); });
  });

  // Docs: copy the commands in a code block, without comment lines.
  Array.prototype.forEach.call(document.querySelectorAll("pre[data-copy]"), function (pre) {
    var wrap = document.createElement("div");
    wrap.className = "has-copy";
    wrap.style.position = "relative";
    pre.parentNode.insertBefore(wrap, pre);
    wrap.appendChild(pre);
    var btn = document.createElement("button");
    btn.type = "button";
    btn.className = "copy-btn";
    btn.textContent = "Copy";
    btn.setAttribute("aria-label", "Copy commands");
    wrap.appendChild(btn);
    btn.addEventListener("click", function () {
      var text = pre.innerText.split("\n").filter(function (line) {
        return line.trim() && line.trim().charAt(0) !== "#";
      }).join("\n");
      var done = function () {
        btn.textContent = "Copied";
        btn.classList.add("done");
        window.setTimeout(function () { btn.textContent = "Copy"; btn.classList.remove("done"); }, 1600);
      };
      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(text).then(done, function () {});
      }
    });
  });

  document.addEventListener("visibilitychange", syncAll);
  if (reduce.addEventListener) reduce.addEventListener("change", syncAll);
  syncAll();
  window.requestAnimationFrame(tick);
})();
