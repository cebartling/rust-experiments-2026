import { listen } from "@tauri-apps/api/event";

interface TranscriptionPayload {
  text: string;
  timestamp: string;
}

export function createTranscriptionLog(container: HTMLElement) {
  const log = document.createElement("div");
  log.className = "transcription-log";
  container.appendChild(log);

  listen<TranscriptionPayload>("transcription-complete", (event) => {
    const entry = document.createElement("div");
    entry.className = "log-entry";
    entry.style.cssText =
      "padding:8px;margin:4px 0;background:#16213e;border-radius:4px;font-size:0.9rem;";
    entry.textContent = event.payload.text;
    log.prepend(entry);
  });

  return {
    clear() {
      log.innerHTML = "";
    },
  };
}
