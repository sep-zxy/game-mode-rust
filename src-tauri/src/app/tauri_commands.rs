use tauri::{AppHandle, State};

use crate::app::tray;
use crate::core::domain::types::{
    ActionListKey, ClashStatus, ConfigV2, ExecutionReport, ProcessInfo,
};
use crate::infra::windows::file_dialog;
use crate::AppState;

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> Result<ConfigV2, String> {
    let mut config = state
        .config_service
        .load_or_init()
        .map_err(|error| error.to_user_message())?;

    if let Ok(enabled) = state.startup_service.is_enabled() {
        if config.global.enable_app_auto_start != enabled {
            config.global.enable_app_auto_start = enabled;
            state
                .config_service
                .save(&config)
                .map_err(|error| error.to_user_message())?;
        }
    }

    Ok(config)
}

#[tauri::command]
pub fn save_config(
    app: AppHandle,
    state: State<'_, AppState>,
    config: ConfigV2,
) -> Result<(), String> {
    state
        .startup_service
        .set_enabled(config.global.enable_app_auto_start)
        .map_err(|error| error.to_user_message())?;

    state
        .mode_service
        .save_config(&state.config_service, &config)
        .map_err(|error| error.to_user_message())?;

    if let Err(error) = tray::create_or_refresh_tray(&app) {
        eprintln!("刷新托盘失败: {}", error.to_user_message());
    }

    Ok(())
}

#[tauri::command]
pub fn enable_mode(app: AppHandle, state: State<'_, AppState>) -> Result<ExecutionReport, String> {
    let report = state
        .mode_service
        .enable_mode(&state.config_service)
        .map_err(|error| error.to_user_message())?;

    if let Err(error) = tray::create_or_refresh_tray(&app) {
        eprintln!("刷新托盘失败: {}", error.to_user_message());
    }

    Ok(report)
}

#[tauri::command]
pub fn disable_mode(app: AppHandle, state: State<'_, AppState>) -> Result<ExecutionReport, String> {
    let report = state
        .mode_service
        .disable_mode(&state.config_service)
        .map_err(|error| error.to_user_message())?;

    if let Err(error) = tray::create_or_refresh_tray(&app) {
        eprintln!("刷新托盘失败: {}", error.to_user_message());
    }

    Ok(report)
}

#[tauri::command]
pub fn switch_active_preset(
    app: AppHandle,
    state: State<'_, AppState>,
    preset_id: String,
) -> Result<(), String> {
    state
        .mode_service
        .switch_active_preset(&state.config_service, &preset_id)
        .map_err(|error| error.to_user_message())?;

    if let Err(error) = tray::create_or_refresh_tray(&app) {
        eprintln!("刷新托盘失败: {}", error.to_user_message());
    }

    Ok(())
}

#[tauri::command]
pub fn list_running_processes(state: State<'_, AppState>) -> Result<Vec<ProcessInfo>, String> {
    state
        .process_service
        .list_running_processes()
        .map_err(|error| error.to_user_message())
}

#[tauri::command]
pub fn test_start_apps(state: State<'_, AppState>, list_key: ActionListKey) -> Result<(), String> {
    state
        .mode_service
        .test_start_apps(&state.config_service, list_key)
        .map_err(|error| error.to_user_message())
}

#[tauri::command]
pub fn test_close_apps(state: State<'_, AppState>, list_key: ActionListKey) -> Result<(), String> {
    state
        .mode_service
        .test_close_apps(&state.config_service, list_key)
        .map_err(|error| error.to_user_message())
}

#[tauri::command]
pub fn set_auto_start(
    app: AppHandle,
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<(), String> {
    state
        .startup_service
        .set_enabled(enabled)
        .map_err(|error| error.to_user_message())?;

    let mut config = state
        .config_service
        .load_or_init()
        .map_err(|error| error.to_user_message())?;
    config.global.enable_app_auto_start = enabled;
    state
        .config_service
        .save(&config)
        .map_err(|error| error.to_user_message())?;

    if let Err(error) = tray::create_or_refresh_tray(&app) {
        eprintln!("刷新托盘失败: {}", error.to_user_message());
    }

    Ok(())
}

#[tauri::command]
pub fn get_clash_status(state: State<'_, AppState>) -> Result<ClashStatus, String> {
    state
        .mode_service
        .get_clash_status(&state.config_service)
        .map_err(|error| error.to_user_message())
}

#[tauri::command]
pub fn select_executable_path() -> Result<Option<String>, String> {
    file_dialog::pick_executable_path().map_err(|error| error.to_user_message())
}

#[tauri::command]
pub fn refresh_tray(app: AppHandle) -> Result<(), String> {
    tray::create_or_refresh_tray(&app).map_err(|error| error.to_user_message())
}
