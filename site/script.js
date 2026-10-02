/* jev-seo proof-sheet terminal. One motion: the line about to change
   flashes amber, then settles into its final ink. Algorave donation. */
(function () {
  "use strict";

  var term = document.getElementById("term");
  if (!term) return;

  var LINES = [
    { t: "$ jev-seo audit docs/ --target-query \"rust grep\"", c: "" },
    { t: "scanning 105 files", c: "dim" },
    { t: "✔ 101 pages passed", c: "ok" },
    { t: "✗ docs/guide.md — title collision with docs/start.md", c: "bad" },
    { t: "✗ docs/v1.md — thin content (214 words < 300)", c: "bad" },
    { t: "✗ blog/seo-vercel.md — orphan (0 inbound links)", c: "bad" },
    { t: "✗ 2 hits of AI vocabulary (\"delve\", \"seamless\")", c: "bad" },
    { t: "→ 4 defects flagged in 0.8s", c: "ok" }
  ];

  var reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  var i = 0, ch = 0, lineEl = null;
  var caret = document.createElement("span");
  caret.className = "blink";
  caret.textContent = "▍";

  function flush() {
    term.textContent = "";
    LINES.forEach(function (l) {
      var s = document.createElement("div");
      if (l.c) s.className = l.c;
      s.textContent = l.t;
      term.appendChild(s);
    });
  }

  function nextLine() {
    if (i >= LINES.length) {
      term.appendChild(caret);
      return;
    }
    var l = LINES[i];
    lineEl = document.createElement("div");
    lineEl.className = "amber"; /* about to change */
    lineEl.textContent = "";
    term.appendChild(lineEl);
    ch = 0;
    typeChar(l);
  }

  function typeChar(l) {
    if (ch < l.t.length) {
      ch++;
      lineEl.textContent = l.t.slice(0, ch);
      setTimeout(function () { typeChar(l); }, 14 + Math.random() * 22);
      return;
    }
    /* line landed -> settle into its final ink */
    lineEl.className = l.c || "";
    i++;
    setTimeout(nextLine, 140);
  }

  if (reduce) { flush(); return; }
  nextLine();
})();