// DevOpster desktop renderer.
// Uses the Tauri v2 global API (no bundler required).

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const log = document.getElementById("log");
const status = document.getElementById("status");
const cmdLabel = document.getElementById("cmd-label");
const exitEl = document.getElementById("exit");

function append(line, cls) {
  const span = document.createElement("span");
  if (cls) span.className = cls;
  span.textContent = line + "\n";
  log.appendChild(span);
  log.scrollTop = log.scrollHeight;
}

function setStatus(text, ok = true) {
  status.textContent = text;
  status.style.color = ok ? "var(--ok)" : "#ff8b91";
}

listen("devopster:stdout", (event) => append(event.payload));
listen("devopster:stderr", (event) => append(event.payload, "err"));

function parseArgs(raw) {
  // very small shell-like splitter: handles quoted strings
  const out = [];
  const re = /"([^"]*)"|'([^']*)'|(\S+)/g;
  let m;
  while ((m = re.exec(raw)) !== null) {
    out.push(m[1] ?? m[2] ?? m[3]);
  }
  return out;
}

async function runStreaming(args, label) {
  cmdLabel.textContent = `$ devopster ${args.join(" ")}`;
  exitEl.textContent = "";
  setStatus("Running…");
  try {
    const code = await invoke("stream_devopster", { args });
    exitEl.textContent = `exit ${code}`;
    setStatus(code === 0 ? "Ready" : "Failed", code === 0);
  } catch (err) {
    append(String(err), "err");
    setStatus("Error", false);
  }
}

document.querySelectorAll(".cmd").forEach((btn) => {
  btn.addEventListener("click", () => {
    const args = JSON.parse(btn.dataset.args);
    runStreaming(args, btn.textContent);
  });
});

document.getElementById("run-custom").addEventListener("click", () => {
  const raw = document.getElementById("custom").value.trim();
  if (!raw) return;
  runStreaming(parseArgs(raw), raw);
});

document.getElementById("custom").addEventListener("keydown", (e) => {
  if (e.key === "Enter") document.getElementById("run-custom").click();
});

document.getElementById("clear").addEventListener("click", () => {
  log.textContent = "";
  exitEl.textContent = "";
  cmdLabel.textContent = "Output cleared.";
});

append("DevOpster desktop ready. Pick a command on the left.", "ok");
