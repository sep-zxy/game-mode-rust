use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionListKey {
    EnableClose,
    EnableStart,
    DisableStart,
    DisableClose,
}

impl ActionListKey {
    pub fn as_field_name(self) -> &'static str {
        match self {
            ActionListKey::EnableClose => "enable_close",
            ActionListKey::EnableStart => "enable_start",
            ActionListKey::DisableStart => "disable_start",
            ActionListKey::DisableClose => "disable_close",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppEntry {
    #[serde(default)]
    pub alias: String,
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub start_args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClashOptions {
    pub enable_manage_clash: bool,
    pub enable_disable_tun: bool,
    pub enable_disable_system_proxy: bool,
    pub disable_manage_clash: bool,
    pub disable_restore_tun: bool,
    pub disable_restore_system_proxy: bool,
    pub disable_start_clash_if_needed: bool,
}

impl Default for ClashOptions {
    fn default() -> Self {
        Self {
            enable_manage_clash: true,
            enable_disable_tun: true,
            enable_disable_system_proxy: true,
            disable_manage_clash: true,
            disable_restore_tun: true,
            disable_restore_system_proxy: true,
            disable_start_clash_if_needed: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub enable_close: Vec<AppEntry>,
    pub enable_start: Vec<AppEntry>,
    pub disable_start: Vec<AppEntry>,
    pub disable_close: Vec<AppEntry>,
    pub clash_options: ClashOptions,
}

impl Default for Preset {
    fn default() -> Self {
        Self {
            id: "preset-default".to_string(),
            name: "默认预设".to_string(),
            enable_close: Vec::new(),
            enable_start: Vec::new(),
            disable_start: Vec::new(),
            disable_close: Vec::new(),
            clash_options: ClashOptions::default(),
        }
    }
}

impl Preset {
    pub fn list(&self, key: ActionListKey) -> &[AppEntry] {
        match key {
            ActionListKey::EnableClose => &self.enable_close,
            ActionListKey::EnableStart => &self.enable_start,
            ActionListKey::DisableStart => &self.disable_start,
            ActionListKey::DisableClose => &self.disable_close,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSettings {
    pub clash_path: String,
    pub clash_port: u16,
    pub clash_secret: String,
    pub enable_app_auto_start: bool,
}

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            clash_path: String::new(),
            clash_port: 9097,
            clash_secret: String::new(),
            enable_app_auto_start: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeState {
    pub mode_active: bool,
    pub last_mode_boot_time: Option<u64>,
    pub last_tun_state: Option<Value>,
    pub last_system_proxy_state: Option<bool>,
    pub windows_proxy_enable: Option<u32>,
    pub windows_proxy_server: Option<String>,
    pub last_error: Option<String>,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            mode_active: false,
            last_mode_boot_time: None,
            last_tun_state: None,
            last_system_proxy_state: None,
            windows_proxy_enable: None,
            windows_proxy_server: None,
            last_error: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigV2 {
    pub version: u32,
    pub global: GlobalSettings,
    pub runtime: RuntimeState,
    pub presets: Vec<Preset>,
    pub active_preset_id: String,
}

impl Default for ConfigV2 {
    fn default() -> Self {
        let preset = Preset::default();
        Self {
            version: 2,
            global: GlobalSettings::default(),
            runtime: RuntimeState::default(),
            active_preset_id: preset.id.clone(),
            presets: vec![preset],
        }
    }
}

impl ConfigV2 {
    pub fn active_preset(&self) -> Option<&Preset> {
        self.presets
            .iter()
            .find(|item| item.id == self.active_preset_id)
            .or_else(|| self.presets.first())
    }

    pub fn active_preset_mut(&mut self) -> Option<&mut Preset> {
        if let Some(index) = self
            .presets
            .iter()
            .position(|item| item.id == self.active_preset_id)
        {
            return self.presets.get_mut(index);
        }
        self.presets.first_mut()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClashStatus {
    pub tun: Option<Value>,
    pub system_proxy: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    pub preset_id: String,
    pub preset_name: String,
    pub mode_active: bool,
    pub executed_actions: Vec<String>,
    pub clash_status: Option<ClashStatus>,
}
