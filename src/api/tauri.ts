import { invoke } from "@tauri-apps/api/core";
import {
  ActionListKey,
  ClashStatus,
  ConfigV2,
  ExecutionReport,
  ProcessInfo,
} from "../types";

export async function getConfig(): Promise<ConfigV2> {
  return invoke<ConfigV2>("get_config");
}

export async function saveConfig(config: ConfigV2): Promise<void> {
  await invoke("save_config", { config });
}

export async function enableMode(): Promise<ExecutionReport> {
  return invoke<ExecutionReport>("enable_mode");
}

export async function disableMode(): Promise<ExecutionReport> {
  return invoke<ExecutionReport>("disable_mode");
}

export async function switchActivePreset(presetId: string): Promise<void> {
  await invoke("switch_active_preset", { presetId });
}

export async function listRunningProcesses(): Promise<ProcessInfo[]> {
  return invoke<ProcessInfo[]>("list_running_processes");
}

export async function testStartApps(listKey: ActionListKey): Promise<void> {
  await invoke("test_start_apps", { listKey });
}

export async function testCloseApps(listKey: ActionListKey): Promise<void> {
  await invoke("test_close_apps", { listKey });
}

export async function setAutoStart(enabled: boolean): Promise<void> {
  await invoke("set_auto_start", { enabled });
}

export async function getClashStatus(): Promise<ClashStatus> {
  return invoke<ClashStatus>("get_clash_status");
}

export async function selectExecutablePath(): Promise<string | null> {
  return invoke<string | null>("select_executable_path");
}

export async function refreshTray(): Promise<void> {
  await invoke("refresh_tray");
}
