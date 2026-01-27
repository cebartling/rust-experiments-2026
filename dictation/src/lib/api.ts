import { invoke } from "@tauri-apps/api/core";

export interface AppSettings {
  hotkey: string;
  sttBackend: "local" | "cloud";
  localModelPath: string;
  localModelSize: string;
  cloudProvider: string;
  cloudApiKey: string;
  cloudModel: string;
  language: string;
  autoInject: boolean;
  notifications: boolean;
  startMinimized: boolean;
  launchAtStartup: boolean;
}

export async function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_settings");
}

export async function updateSettings(
  settings: Partial<AppSettings>,
): Promise<void> {
  return invoke("update_settings", { settings });
}

export async function testSttBackend(): Promise<string> {
  return invoke<string>("test_stt_backend");
}

export async function getAudioDevices(): Promise<string[]> {
  return invoke<string[]>("get_audio_devices");
}
