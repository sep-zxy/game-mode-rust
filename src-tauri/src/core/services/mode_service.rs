use std::sync::Arc;

use sysinfo::System;

use crate::core::domain::error::{AppError, AppResult};
use crate::core::domain::types::{ActionListKey, ClashStatus, ConfigV2, ExecutionReport};

use super::clash_service::ClashService;
use super::config_service::ConfigService;
use super::process_service::ProcessService;

pub struct ModeService {
    process_service: Arc<ProcessService>,
    clash_service: Arc<ClashService>,
}

impl ModeService {
    pub fn new(process_service: Arc<ProcessService>, clash_service: Arc<ClashService>) -> Self {
        Self {
            process_service,
            clash_service,
        }
    }

    pub fn enable_mode(&self, config_service: &ConfigService) -> AppResult<ExecutionReport> {
        let mut config = config_service.load_or_init()?;
        if config.runtime.mode_active {
            return Err(AppError::Conflict("当前预设已处于开启状态".to_string()));
        }

        let preset = config
            .active_preset()
            .cloned()
            .ok_or_else(|| AppError::NotFound("找不到当前激活预设".to_string()))?;

        let mut actions = Vec::new();

        if preset.clash_options.enable_manage_clash {
            actions.extend(self.clash_service.disable_and_capture(
                &config.global,
                &mut config.runtime,
                &preset.clash_options,
            )?);
        }

        self.process_service.close_all(&preset.enable_close)?;
        if !preset.enable_close.is_empty() {
            actions.push("close enable_close apps".to_string());
        }

        self.process_service.start_all(&preset.enable_start)?;
        if !preset.enable_start.is_empty() {
            actions.push("start enable_start apps".to_string());
        }

        config.runtime.mode_active = true;
        config.runtime.last_mode_boot_time = Some(System::boot_time());
        config.runtime.last_error = None;
        config_service.save(&config)?;

        let clash_status = self.clash_service.get_status(&config.global).ok();

        Ok(ExecutionReport {
            preset_id: preset.id,
            preset_name: preset.name,
            mode_active: true,
            executed_actions: actions,
            clash_status,
        })
    }

    pub fn disable_mode(&self, config_service: &ConfigService) -> AppResult<ExecutionReport> {
        let mut config = config_service.load_or_init()?;
        if !config.runtime.mode_active {
            return Err(AppError::Conflict("当前预设已处于关闭状态".to_string()));
        }

        let preset = config
            .active_preset()
            .cloned()
            .ok_or_else(|| AppError::NotFound("找不到当前激活预设".to_string()))?;

        let mut actions = Vec::new();

        if preset.clash_options.disable_manage_clash {
            actions.extend(self.clash_service.restore(
                &config.global,
                &config.runtime,
                &preset.clash_options,
            )?);
        }

        self.process_service.close_all(&preset.disable_close)?;
        if !preset.disable_close.is_empty() {
            actions.push("close disable_close apps".to_string());
        }

        self.process_service.start_all(&preset.disable_start)?;
        if !preset.disable_start.is_empty() {
            actions.push("start disable_start apps".to_string());
        }

        config.runtime.mode_active = false;
        config.runtime.last_mode_boot_time = None;
        config.runtime.last_error = None;
        config_service.save(&config)?;

        let clash_status = self.clash_service.get_status(&config.global).ok();

        Ok(ExecutionReport {
            preset_id: preset.id,
            preset_name: preset.name,
            mode_active: false,
            executed_actions: actions,
            clash_status,
        })
    }

    pub fn switch_active_preset(
        &self,
        config_service: &ConfigService,
        preset_id: &str,
    ) -> AppResult<()> {
        let mut config = config_service.load_or_init()?;
        if config.runtime.mode_active {
            return Err(AppError::Conflict(
                "当前处于已开启状态，需先关闭当前预设".to_string(),
            ));
        }

        if !config.presets.iter().any(|preset| preset.id == preset_id) {
            return Err(AppError::NotFound(format!("目标预设不存在: {preset_id}")));
        }

        config.active_preset_id = preset_id.to_string();
        config_service.save(&config)
    }

    pub fn test_start_apps(
        &self,
        config_service: &ConfigService,
        list_key: ActionListKey,
    ) -> AppResult<()> {
        let config = config_service.load_or_init()?;
        let preset = config
            .active_preset()
            .ok_or_else(|| AppError::NotFound("找不到当前激活预设".to_string()))?;

        self.process_service.start_all(preset.list(list_key))
    }

    pub fn test_close_apps(
        &self,
        config_service: &ConfigService,
        list_key: ActionListKey,
    ) -> AppResult<()> {
        let config = config_service.load_or_init()?;
        let preset = config
            .active_preset()
            .ok_or_else(|| AppError::NotFound("找不到当前激活预设".to_string()))?;

        self.process_service.close_all(preset.list(list_key))
    }

    pub fn get_clash_status(&self, config_service: &ConfigService) -> AppResult<ClashStatus> {
        let config = config_service.load_or_init()?;
        self.clash_service.get_status(&config.global)
    }

    pub fn reset_mode_after_boot(&self, config_service: &ConfigService) -> AppResult<bool> {
        let mut config = config_service.load_or_init()?;
        if !config.runtime.mode_active {
            return Ok(false);
        }

        let current_boot_time = System::boot_time();
        if config.runtime.last_mode_boot_time == Some(current_boot_time) {
            return Ok(false);
        }

        config.runtime.mode_active = false;
        config.runtime.last_mode_boot_time = None;
        config.runtime.last_tun_state = None;
        config.runtime.last_system_proxy_state = None;
        config.runtime.windows_proxy_enable = None;
        config.runtime.windows_proxy_server = None;
        config.runtime.last_error = Some("检测到系统重启，已自动退出上次游戏模式".to_string());
        config_service.save(&config)?;

        Ok(true)
    }

    pub fn save_config(&self, config_service: &ConfigService, config: &ConfigV2) -> AppResult<()> {
        config_service.save(config)
    }
}
