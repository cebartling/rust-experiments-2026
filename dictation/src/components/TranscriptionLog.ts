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

    const ts = document.createElement("div");
    ts.className = "timestamp";
    const date = new Date(Number(event.payload.timestamp));
    ts.textContent = date.toLocaleTimeString();
    entry.appendChild(ts);

    const text = document.createElement("div");
    text.textContent = event.payload.text;
    entry.appendChild(text);

    log.prepend(entry);
  });

  return {
    clear() {
      log.innerHTML = "";
    },
  };
}
