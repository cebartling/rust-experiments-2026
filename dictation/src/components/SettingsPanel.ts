import {
  getSettings,
  updateSettings,
  testSttBackend,
  type AppSettings,
} from "../lib/api";

export async function createSettingsPanel(container: HTMLElement) {
  let settings = await getSettings();

  const panel = document.createElement("div");
  panel.className = "settings-panel";
  container.appendChild(panel);

  function render() {
    panel.innerHTML = `
      <h2>Settings</h2>

      <div class="form-group">
        <label for="setting-hotkey">Push-to-talk hotkey</label>
        <input type="text" id="setting-hotkey" value="${settings.hotkey}" />
      </div>

      <div class="form-group">
        <label for="setting-language">Language (ISO 639-1)</label>
        <input type="text" id="setting-language" value="${settings.language}" />
      </div>

      <h3>STT Backend</h3>

      <div class="form-group">
        <label for="setting-backend">Backend</label>
        <select id="setting-backend">
          <option value="local" ${settings.sttBackend === "local" ? "selected" : ""}>Local (Whisper)</option>
          <option value="cloud" ${settings.sttBackend === "cloud" ? "selected" : ""}>Cloud (OpenAI)</option>
        </select>
      </div>

      <div id="local-settings" class="${settings.sttBackend === "local" ? "" : "hidden"}">
        <div class="form-group">
          <label for="setting-model-path">Model file path</label>
          <input type="text" id="setting-model-path" value="${settings.localModelPath}" placeholder="/path/to/ggml-base.en.bin" />
        </div>
        <div class="form-group">
          <label for="setting-model-size">Model size</label>
          <select id="setting-model-size">
            ${(["tiny", "base", "small", "medium", "large"] as const)
              .map(
                (s) =>
                  `<option value="${s}" ${settings.localModelSize === s ? "selected" : ""}>${s}</option>`,
              )
              .join("")}
          </select>
        </div>
      </div>

      <div id="cloud-settings" class="${settings.sttBackend === "cloud" ? "" : "hidden"}">
        <div class="form-group">
          <label for="setting-cloud-provider">Provider</label>
          <input type="text" id="setting-cloud-provider" value="${settings.cloudProvider}" />
        </div>
        <div class="form-group">
          <label for="setting-cloud-api-key">API key</label>
          <input type="text" id="setting-cloud-api-key" value="${settings.cloudApiKey}" placeholder="sk-..." />
        </div>
        <div class="form-group">
          <label for="setting-cloud-model">Model</label>
          <input type="text" id="setting-cloud-model" value="${settings.cloudModel}" />
        </div>
      </div>

      <div style="margin-top:12px">
        <button class="btn" id="btn-test-backend">Test Backend</button>
        <div id="test-result" class="test-result hidden"></div>
      </div>

      <h3>Behavior</h3>

      <div class="form-row" style="margin-bottom:8px">
        <label><input type="checkbox" id="setting-auto-inject" ${settings.autoInject ? "checked" : ""} /> Auto-inject text</label>
      </div>
      <div class="form-row" style="margin-bottom:8px">
        <label><input type="checkbox" id="setting-notifications" ${settings.notifications ? "checked" : ""} /> Notifications</label>
      </div>
      <div class="form-row" style="margin-bottom:8px">
        <label><input type="checkbox" id="setting-start-minimized" ${settings.startMinimized ? "checked" : ""} /> Start minimized</label>
      </div>
      <div class="form-row" style="margin-bottom:8px">
        <label><input type="checkbox" id="setting-launch-startup" ${settings.launchAtStartup ? "checked" : ""} /> Launch at startup</label>
      </div>

      <div style="margin-top:16px">
        <button class="btn btn-primary" id="btn-save-settings">Save Settings</button>
      </div>
    `;

    bindEvents();
  }

  function bindEvents() {
    // Toggle local/cloud sections when backend changes
    const backendSelect = panel.querySelector(
      "#setting-backend",
    ) as HTMLSelectElement;
    backendSelect.addEventListener("change", () => {
      const localSection = panel.querySelector(
        "#local-settings",
      ) as HTMLElement;
      const cloudSection = panel.querySelector(
        "#cloud-settings",
      ) as HTMLElement;
      if (backendSelect.value === "local") {
        localSection.classList.remove("hidden");
        cloudSection.classList.add("hidden");
      } else {
        localSection.classList.add("hidden");
        cloudSection.classList.remove("hidden");
      }
    });

    // Test backend button
    const testBtn = panel.querySelector(
      "#btn-test-backend",
    ) as HTMLButtonElement;
    testBtn.addEventListener("click", async () => {
      const resultEl = panel.querySelector("#test-result") as HTMLElement;
      resultEl.classList.remove("hidden", "success", "error");
      resultEl.textContent = "Testing...";

      // Apply current form values before testing
      await saveCurrentSettings();

      try {
        const msg = await testSttBackend();
        resultEl.classList.add("success");
        resultEl.textContent = msg;
      } catch (err) {
        resultEl.classList.add("error");
        resultEl.textContent = String(err);
      }
    });

    // Save button
    const saveBtn = panel.querySelector(
      "#btn-save-settings",
    ) as HTMLButtonElement;
    saveBtn.addEventListener("click", async () => {
      await saveCurrentSettings();
      saveBtn.textContent = "Saved";
      setTimeout(() => {
        saveBtn.textContent = "Save Settings";
      }, 1500);
    });
  }

  async function saveCurrentSettings() {
    const getValue = (id: string) =>
      (panel.querySelector(`#${id}`) as HTMLInputElement).value;
    const getChecked = (id: string) =>
      (panel.querySelector(`#${id}`) as HTMLInputElement).checked;

    settings = {
      hotkey: getValue("setting-hotkey"),
      sttBackend: getValue("setting-backend") as "local" | "cloud",
      localModelPath: getValue("setting-model-path"),
      localModelSize: getValue("setting-model-size") as AppSettings["localModelSize"],
      cloudProvider: getValue("setting-cloud-provider"),
      cloudApiKey: getValue("setting-cloud-api-key"),
      cloudModel: getValue("setting-cloud-model"),
      language: getValue("setting-language"),
      autoInject: getChecked("setting-auto-inject"),
      notifications: getChecked("setting-notifications"),
      startMinimized: getChecked("setting-start-minimized"),
      launchAtStartup: getChecked("setting-launch-startup"),
    };

    await updateSettings(settings);
  }

  render();
}
