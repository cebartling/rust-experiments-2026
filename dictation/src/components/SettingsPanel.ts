import { getSettings, updateSettings, type AppSettings } from "../lib/api";

export async function createSettingsPanel(container: HTMLElement) {
  const settings = await getSettings();

  const panel = document.createElement("div");
  panel.className = "settings-panel";
  panel.innerHTML = `
    <h2>Settings</h2>
    <label>
      Hotkey: <input type="text" id="setting-hotkey" value="${settings.hotkey}" />
    </label>
    <label>
      Backend:
      <select id="setting-backend">
        <option value="local" ${settings.sttBackend === "local" ? "selected" : ""}>Local (Whisper)</option>
        <option value="cloud" ${settings.sttBackend === "cloud" ? "selected" : ""}>Cloud</option>
      </select>
    </label>
    <label>
      Language: <input type="text" id="setting-language" value="${settings.language}" />
    </label>
  `;
  container.appendChild(panel);

  panel.addEventListener("change", async () => {
    const updated: Partial<AppSettings> = {
      hotkey: (document.getElementById("setting-hotkey") as HTMLInputElement)
        .value,
      sttBackend: (
        document.getElementById("setting-backend") as HTMLSelectElement
      ).value as "local" | "cloud",
      language: (
        document.getElementById("setting-language") as HTMLInputElement
      ).value,
    };
    await updateSettings(updated);
  });
}
