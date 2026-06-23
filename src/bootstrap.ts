import { invoke } from "@tauri-apps/api/core";

function report(message: string, show = false) {
  invoke("report_frontend_error", { message }).catch(() => {});
  const root = document.getElementById("root");
  if (show && root) root.innerHTML = `<div style="padding:32px;color:#fecaca;font:16px Segoe UI,sans-serif"><h1>Application error</h1><pre style="white-space:pre-wrap">${message.replace(/[&<>]/g, c => ({"&":"&amp;","<":"&lt;",">":"&gt;"}[c]!))}</pre></div>`;
}

window.addEventListener("error", event => report(`${event.message}\n${event.error?.stack ?? ""}`));
window.addEventListener("unhandledrejection", event => report(`Unhandled promise rejection: ${event.reason?.stack ?? event.reason}`));
import("./main").catch(error => report(`Failed to load interface: ${error?.stack ?? error}`, true));
