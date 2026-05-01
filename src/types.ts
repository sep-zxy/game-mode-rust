export type ActionListKey = "enable_close" | "enable_start" | "disable_start" | "disable_close";

export interface AppEntry {
  alias: string;
  name: string;
  path: string;
  start_args: string[];
}

export interface ClashOptions {
  enable_manage_clash: boolean;
  enable_disable_tun: boolean;
  enable_disable_system_proxy: boolean;
  disable_manage_clash: boolean;
  disable_restore_tun: boolean;
  disable_restore_system_proxy: boolean;
  disable_start_clash_if_needed: boolean;
}

export interface Preset {
  id: string;
  name: string;
  enable_close: AppEntry[];
  enable_start: AppEntry[];
  disable_start: AppEntry[];
  disable_close: AppEntry[];
  clash_options: ClashOptions;
}

export interface GlobalSettings {
  clash_path: string;
  clash_port: number;
  clash_secret: string;
  enable_app_auto_start: boolean;
}

export interface RuntimeState {
  mode_active: boolean;
  last_mode_boot_time: number | null;
  last_tun_state: unknown | null;
  last_system_proxy_state: boolean | null;
  windows_proxy_enable: number | null;
  windows_proxy_server: string | null;
  last_error: string | null;
}

export interface ConfigV2 {
  version: number;
  global: GlobalSettings;
  runtime: RuntimeState;
  presets: Preset[];
  active_preset_id: string;
}

export interface ClashStatus {
  tun: unknown | null;
  system_proxy: boolean | null;
}

export interface ExecutionReport {
  preset_id: string;
  preset_name: string;
  mode_active: boolean;
  executed_actions: string[];
  clash_status: ClashStatus | null;
}

export interface ProcessInfo {
  name: string;
  path: string;
}
