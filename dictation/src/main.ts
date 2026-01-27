import { listen } from "@tauri-apps/api/event";

interface DictationStatePayload {
  state: "idle" | "recording" | "transcribing" | "error";
  message?: string;
}

interface TranscriptionPayload {
  text: string;
  timestamp: string;
}

const statusEl = document.getElementById("status")!;
const logEl = document.getElementById("log")!;

function setStatus(state: DictationStatePayload["state"], message?: string) {
  statusEl.className = `status ${state}`;
  const labels: Record<string, string> = {
    idle: "Idle",
    recording: "Recording...",
    transcribing: "Transcribing...",
    error: `Error: ${message ?? "unknown"}`,
  };
  statusEl.textContent = labels[state] ?? state;
}

listen<DictationStatePayload>("dictation-state-changed", (event) => {
  setStatus(event.payload.state, event.payload.message);
});

listen<TranscriptionPayload>("transcription-complete", (event) => {
  const entry = document.createElement("div");
  entry.style.cssText =
    "padding:8px;margin:4px 0;background:#16213e;border-radius:4px;font-size:0.9rem;";
  entry.textContent = event.payload.text;
  logEl.prepend(entry);
});

listen<{ message: string }>("dictation-error", (event) => {
  setStatus("error", event.payload.message);
});
