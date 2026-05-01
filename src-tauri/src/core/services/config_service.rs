use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use chrono::Local;

use crate::core::domain::error::{AppError, AppResult};
use crate::core::domain::types::ConfigV2;

pub struct ConfigService {
    config_path: PathBuf,
    lock: Mutex<()>,
}

impl ConfigService {
    pub fn new() -> Self {
        let config_path = resolve_config_path();
        Self {
            config_path,
            lock: Mutex::new(()),
        }
    }

    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    pub fn load_or_init(&self) -> AppResult<ConfigV2> {
        let _guard = self
            .lock
            .lock()
            .map_err(|_| AppError::System("配置锁获取失败".to_string()))?;
        self.load_or_init_unlocked()
    }

    pub fn save(&self, config: &ConfigV2) -> AppResult<()> {
        let _guard = self
            .lock
            .lock()
            .map_err(|_| AppError::System("配置锁获取失败".to_string()))?;
        self.save_unlocked(config)
    }

    fn load_or_init_unlocked(&self) -> AppResult<ConfigV2> {
        if !self.config_path.exists() {
            let default = ConfigV2::default();
            self.save_unlocked(&default)?;
            return Ok(default);
        }

        let text = fs::read_to_string(&self.config_path)?;
        match serde_json::from_str::<ConfigV2>(&text) {
            Ok(mut config) => {
                normalize_config(&mut config);
                validate_config(&config)?;
                Ok(config)
            }
            Err(_error) => {
                self.backup_invalid_config(&text)?;
                let default = ConfigV2::default();
                self.save_unlocked(&default)?;
                Ok(default)
            }
        }
    }

    fn save_unlocked(&self, config: &ConfigV2) -> AppResult<()> {
        let mut normalized = config.clone();
        normalize_config(&mut normalized);
        validate_config(&normalized)?;

        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let payload = serde_json::to_vec_pretty(&normalized)?;
        let tmp_path = self
            .config_path
            .with_extension(format!("tmp.{}", std::process::id()));

        fs::write(&tmp_path, payload)?;

        if self.config_path.exists() {
            fs::remove_file(&self.config_path)?;
        }
        fs::rename(&tmp_path, &self.config_path)?;
        Ok(())
    }

    fn backup_invalid_config(&self, content: &str) -> AppResult<()> {
        let parent = self
            .config_path
            .parent()
            .ok_or_else(|| AppError::Config("配置目录解析失败".to_string()))?;
        fs::create_dir_all(parent)?;

        let stamp = Local::now().format("%Y%m%d-%H%M%S");
        let backup_path = parent.join(format!("config_v2.invalid-{stamp}.json"));
        fs::write(backup_path, content.as_bytes())?;
        Ok(())
    }
}

fn resolve_config_path() -> PathBuf {
    let appdata = std::env::var("APPDATA").ok();
    let mut base = appdata
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    base.push("GameModeSwitcherRust");
    base.push("config_v2.json");
    base
}

fn normalize_config(config: &mut ConfigV2) {
    if config.version != 2 {
        config.version = 2;
    }

    if config.presets.is_empty() {
        config
            .presets
            .push(crate::core::domain::types::Preset::default());
    }

    for preset in &mut config.presets {
        for entry in preset
            .enable_close
            .iter_mut()
            .chain(preset.enable_start.iter_mut())
            .chain(preset.disable_start.iter_mut())
            .chain(preset.disable_close.iter_mut())
        {
            if entry.alias.trim().is_empty() {
                entry.alias = entry.name.clone();
            }
        }

        preset
            .enable_close
            .retain(|item| !item.name.trim().is_empty() && !item.path.trim().is_empty());
        preset
            .enable_start
            .retain(|item| !item.name.trim().is_empty() && !item.path.trim().is_empty());
        preset
            .disable_start
            .retain(|item| !item.name.trim().is_empty() && !item.path.trim().is_empty());
        preset
            .disable_close
            .retain(|item| !item.name.trim().is_empty() && !item.path.trim().is_empty());
    }

    if !config
        .presets
        .iter()
        .any(|preset| preset.id == config.active_preset_id)
    {
        if let Some(first) = config.presets.first() {
            config.active_preset_id = first.id.clone();
        }
    }
}

fn validate_config(config: &ConfigV2) -> AppResult<()> {
    if config.presets.is_empty() {
        return Err(AppError::Validation("至少保留一个预设".to_string()));
    }

    let mut seen = HashSet::new();
    for preset in &config.presets {
        if preset.id.trim().is_empty() {
            return Err(AppError::Validation("预设 ID 不能为空".to_string()));
        }
        if !seen.insert(preset.id.to_lowercase()) {
            return Err(AppError::Validation(format!("重复预设 ID: {}", preset.id)));
        }
        if preset.name.trim().is_empty() {
            return Err(AppError::Validation(format!(
                "预设名称不能为空: {}",
                preset.id
            )));
        }

        for entry in preset
            .enable_close
            .iter()
            .chain(preset.enable_start.iter())
            .chain(preset.disable_start.iter())
            .chain(preset.disable_close.iter())
        {
            if entry.name.trim().is_empty() || entry.path.trim().is_empty() {
                return Err(AppError::Validation("应用 name/path 不能为空".to_string()));
            }
        }
    }

    if !config
        .presets
        .iter()
        .any(|preset| preset.id == config.active_preset_id)
    {
        return Err(AppError::Validation("active_preset_id 无效".to_string()));
    }

    if config.global.clash_port == 0 {
        return Err(AppError::Validation(
            "clash_port 必须在 1-65535".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_defaults() {
        let cfg = ConfigV2::default();
        validate_config(&cfg).expect("default config should be valid");
    }
}
