pub mod app;
pub mod core;
pub mod infra;

use std::sync::Arc;

use app::tauri_commands;
use app::tray;
use core::services::clash_service::ClashService;
use core::services::config_service::ConfigService;
use core::services::mode_service::ModeService;
use core::services::process_service::ProcessService;
use core::services::startup_service::StartupService;
use infra::windows::elevation;
use tauri::Manager;

#[derive(Clone)]
pub struct AppState {
    pub config_service: Arc<ConfigService>,
    pub process_service: Arc<ProcessService>,
    pub startup_service: Arc<StartupService>,
    pub clash_service: Arc<ClashService>,
    pub mode_service: Arc<ModeService>,
}

pub fn bootstrap_and_run() {
    let args = std::env::args().collect::<Vec<_>>();
    let startup_silent = args
        .iter()
        .skip(1)
        .any(|arg| arg.eq_ignore_ascii_case("--startup-silent"));
    let no_elevate = args
        .iter()
        .skip(1)
        .any(|arg| arg.eq_ignore_ascii_case("--no-elevate"));
    let startup_silent_on_boot = startup_silent;

    if !startup_silent && !no_elevate && !elevation::is_admin() {
        if let Err(error) = elevation::relaunch_as_admin(&args) {
            eprintln!("管理员提权失败: {}", error.to_user_message());
        }
        return;
    }

    let config_service = Arc::new(ConfigService::new());
    let process_service = Arc::new(ProcessService::new());
    let startup_service = Arc::new(StartupService::new());
    let clash_service = Arc::new(ClashService::new());
    let mode_service = Arc::new(ModeService::new(
        process_service.clone(),
        clash_service.clone(),
    ));

    let state = AppState {
        config_service,
        process_service,
        startup_service,
        clash_service,
        mode_service,
    };

    tauri::Builder::default()
        .manage(state)
        .setup(move |app| {
            let state = app.state::<AppState>();

            if let Err(error) = state
                .mode_service
                .reset_mode_after_boot(&state.config_service)
            {
                eprintln!("开机重置游戏模式失败: {}", error.to_user_message());

                if let Ok(mut config) = state.config_service.load_or_init() {
                    config.runtime.last_error =
                        Some(format!("开机重置失败: {}", error.to_user_message()));
                    let _ = state.config_service.save(&config);
                }
            }

            tray::create_or_refresh_tray(app.handle())?;

            let mut hide_window = startup_silent_on_boot;
            if !hide_window {
                if let Ok(config) = state.config_service.load_or_init() {
                    if let Some(active_preset) = config.active_preset() {
                        hide_window = !active_preset.enable_close.is_empty()
                            || !active_preset.enable_start.is_empty()
                            || !active_preset.disable_start.is_empty()
                            || !active_preset.disable_close.is_empty();
                    }
                }
            }

            if hide_window {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            tauri_commands::get_config,
            tauri_commands::save_config,
            tauri_commands::enable_mode,
            tauri_commands::disable_mode,
            tauri_commands::switch_active_preset,
            tauri_commands::list_running_processes,
            tauri_commands::test_start_apps,
            tauri_commands::test_close_apps,
            tauri_commands::set_auto_start,
            tauri_commands::get_clash_status,
            tauri_commands::select_executable_path,
            tauri_commands::refresh_tray,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|error| panic!("Tauri 应用启动失败: {error}"));
}
