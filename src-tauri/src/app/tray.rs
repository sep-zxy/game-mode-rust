use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

use crate::core::domain::error::{AppError, AppResult};
use crate::core::domain::types::ConfigV2;
use crate::AppState;

const TRAY_ID: &str = "main";

pub fn create_or_refresh_tray(app: &AppHandle) -> AppResult<()> {
    let state = app.state::<AppState>();
    let config = state.config_service.load_or_init()?;
    let menu = build_tray_menu(app, &config)?;
    let tooltip = build_tooltip(&config);

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        tray.set_menu(Some(menu))
            .map_err(|error| AppError::System(format!("更新托盘菜单失败: {error}")))?;
        tray.set_tooltip(Some(&tooltip))
            .map_err(|error| AppError::System(format!("更新托盘提示失败: {error}")))?;
        return Ok(());
    }

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .tooltip(&tooltip);

    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }

    builder
        .on_menu_event(|app, event| {
            let menu_id = event.id().as_ref().to_string();
            if let Err(error) = handle_tray_menu_click(app, &menu_id) {
                eprintln!("托盘菜单处理失败: {}", error.to_user_message());
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)
        .map_err(|error| AppError::System(format!("创建托盘图标失败: {error}")))?;

    Ok(())
}

fn build_tray_menu(app: &AppHandle, config: &ConfigV2) -> AppResult<Menu<tauri::Wry>> {
    let active_preset = config
        .active_preset()
        .ok_or_else(|| AppError::NotFound("找不到当前激活预设".to_string()))?;

    let is_active = config.runtime.mode_active;
    let enable_text = format!("开启当前预设（{}）", active_preset.name);
    let disable_text = format!("关闭当前预设（{}）", active_preset.name);

    let menu =
        Menu::new(app).map_err(|error| AppError::System(format!("创建托盘菜单失败: {error}")))?;

    let enable_item = MenuItem::with_id(app, "enable_mode", enable_text, !is_active, None::<&str>)
        .map_err(|error| AppError::System(format!("创建菜单项失败: {error}")))?;
    menu.append(&enable_item)
        .map_err(|error| AppError::System(format!("写入菜单失败: {error}")))?;

    let disable_item =
        MenuItem::with_id(app, "disable_mode", disable_text, is_active, None::<&str>)
            .map_err(|error| AppError::System(format!("创建菜单项失败: {error}")))?;
    menu.append(&disable_item)
        .map_err(|error| AppError::System(format!("写入菜单失败: {error}")))?;

    let sep = PredefinedMenuItem::separator(app)
        .map_err(|error| AppError::System(format!("创建分割线失败: {error}")))?;
    menu.append(&sep)
        .map_err(|error| AppError::System(format!("写入菜单失败: {error}")))?;

    for preset in &config.presets {
        let checked = preset.id == config.active_preset_id;
        let marker = if checked { "●" } else { "○" };
        let label = format!("{marker} 切换预设: {}", preset.name);
        let id = format!("switch_preset::{}", preset.id);
        let item = MenuItem::with_id(app, id, label, !is_active, None::<&str>)
            .map_err(|error| AppError::System(format!("创建菜单项失败: {error}")))?;
        menu.append(&item)
            .map_err(|error| AppError::System(format!("写入菜单失败: {error}")))?;
    }

    let sep2 = PredefinedMenuItem::separator(app)
        .map_err(|error| AppError::System(format!("创建分割线失败: {error}")))?;
    menu.append(&sep2)
        .map_err(|error| AppError::System(format!("写入菜单失败: {error}")))?;

    let settings_item = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)
        .map_err(|error| AppError::System(format!("创建菜单项失败: {error}")))?;
    menu.append(&settings_item)
        .map_err(|error| AppError::System(format!("写入菜单失败: {error}")))?;

    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)
        .map_err(|error| AppError::System(format!("创建菜单项失败: {error}")))?;
    menu.append(&quit_item)
        .map_err(|error| AppError::System(format!("写入菜单失败: {error}")))?;

    Ok(menu)
}

fn build_tooltip(config: &ConfigV2) -> String {
    let preset_name = config
        .active_preset()
        .map(|preset| preset.name.as_str())
        .unwrap_or("-");
    let mode_text = if config.runtime.mode_active {
        "模式已开启"
    } else {
        "模式已关闭"
    };
    format!("GameMode Switcher Rust\n{mode_text}\n当前预设: {preset_name}")
}

fn handle_tray_menu_click(app: &AppHandle, menu_id: &str) -> AppResult<()> {
    let state = app.state::<AppState>();

    match menu_id {
        "enable_mode" => {
            let _ = state.mode_service.enable_mode(&state.config_service)?;
            create_or_refresh_tray(app)?;
        }
        "disable_mode" => {
            let _ = state.mode_service.disable_mode(&state.config_service)?;
            create_or_refresh_tray(app)?;
        }
        "settings" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "quit" => {
            app.exit(0);
        }
        _ if menu_id.starts_with("switch_preset::") => {
            let preset_id = menu_id.trim_start_matches("switch_preset::");
            state
                .mode_service
                .switch_active_preset(&state.config_service, preset_id)?;
            create_or_refresh_tray(app)?;
        }
        _ => {}
    }

    Ok(())
}
