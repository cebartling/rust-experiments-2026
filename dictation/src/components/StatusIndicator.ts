export type DictationState = "idle" | "recording" | "transcribing" | "error";

export function createStatusIndicator(container: HTMLElement) {
  const el = document.createElement("div");
  el.className = "status idle";
  el.textContent = "Idle";
  container.appendChild(el);

  return {
    update(state: DictationState, message?: string) {
      el.className = `status ${state}`;
      const labels: Record<DictationState, string> = {
        idle: "Idle",
        recording: "Recording...",
        transcribing: "Transcribing...",
        error: `Error: ${message ?? "unknown"}`,
      };
      el.textContent = labels[state];
    },
  };
}
