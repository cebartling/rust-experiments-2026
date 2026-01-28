import { listen } from "@tauri-apps/api/event";
import { createStatusIndicator, type DictationState } from "./components/StatusIndicator";
import { createSettingsPanel } from "./components/SettingsPanel";
import { createTranscriptionLog } from "./components/TranscriptionLog";

interface DictationStatePayload {
  state: DictationState;
  message?: string;
}

// Initialize components
const statusContainer = document.getElementById("status-indicator")!;
const logContainer = document.getElementById("tab-log")!;
const settingsContainer = document.getElementById("tab-settings")!;

const status = createStatusIndicator(statusContainer);
createTranscriptionLog(logContainer);
createSettingsPanel(settingsContainer);

// Listen for state changes from the Rust backend
listen<DictationStatePayload>("dictation-state-changed", (event) => {
  status.update(event.payload.state, event.payload.message);
});

listen<{ message: string }>("dictation-error", (event) => {
  status.update("error", event.payload.message);
});

// Tab navigation
document.querySelectorAll<HTMLButtonElement>(".tab-btn").forEach((btn) => {
  btn.addEventListener("click", () => {
    document.querySelectorAll(".tab-btn").forEach((b) => b.classList.remove("active"));
    document.querySelectorAll(".tab-content").forEach((c) => c.classList.remove("active"));
    btn.classList.add("active");
    const target = document.getElementById(btn.dataset.tab!);
    if (target) target.classList.add("active");
  });
});
