use std::fs;
use std::path::{Path, PathBuf};

use sysinfo::{ProcessesToUpdate, Signal, System};

use crate::core::domain::error::{AppError, AppResult};
use crate::core::domain::types::{ClashOptions, ClashStatus, GlobalSettings, RuntimeState};
use crate::infra::clash::client::{extract_tun_enabled, ClashClient};
use crate::infra::windows::{process as windows_process, proxy, wininet};

#[derive(Default)]
pub struct ClashService;

impl ClashService {
    pub fn new() -> Self {
        Self
    }

    pub fn get_status(&self, global: &GlobalSettings) -> AppResult<ClashStatus> {
        let client = ClashClient::from_global(global)?;
        client.get_status()
    }

    pub fn disable_and_capture(
        &self,
        global: &GlobalSettings,
        runtime: &mut RuntimeState,
        options: &ClashOptions,
    ) -> AppResult<Vec<String>> {
        let mut actions = Vec::new();
        let client = ClashClient::from_global(global)?;

        if options.enable_disable_tun || options.enable_disable_system_proxy {
            let current = client.get_status()?;
            runtime.last_tun_state = current.tun.clone();
            runtime.last_system_proxy_state = current.system_proxy;
            actions.push("capture current clash states".to_string());
        }

        if options.enable_disable_tun {
            client.set_proxy(false, Some(false), 3)?;
            self.sync_verge_state_flags(Some(false), None)?;
            actions.push("disable tun".to_string());
        }

        if options.enable_disable_system_proxy {
            proxy::disable_proxy(runtime)?;
            wininet::refresh_internet_options()?;
            self.sync_verge_state_flags(None, Some(false))?;
            actions.push("disable windows system proxy".to_string());
        }

        let _ = self.restart_verge_ui_for_display_sync(global);
        Ok(actions)
    }

    pub fn restore(
        &self,
        global: &GlobalSettings,
        runtime: &RuntimeState,
        options: &ClashOptions,
    ) -> AppResult<Vec<String>> {
        let mut actions = Vec::new();

        if options.disable_start_clash_if_needed {
            self.start_clash_if_needed(global)?;
            actions.push("ensure clash verge process".to_string());
        }

        if options.disable_restore_system_proxy {
            proxy::restore_proxy(runtime)?;
            wininet::refresh_internet_options()?;
            let target = runtime
                .windows_proxy_enable
                .map(|value| value != 0)
                .or(runtime.last_system_proxy_state)
                .unwrap_or(false);
            self.sync_verge_state_flags(None, Some(target))?;
            actions.push("restore windows system proxy".to_string());
        }

        if options.disable_restore_tun {
            let client = ClashClient::from_global(global)?;
            let target_tun = extract_tun_enabled(runtime.last_tun_state.as_ref());
            let target_sys = runtime.last_system_proxy_state;
            let mut last_error = String::new();

            for _ in 0..20 {
                std::thread::sleep(std::time::Duration::from_secs(1));
                match client.set_proxy(target_tun, target_sys, 1) {
                    Ok(_) => {
                        self.sync_verge_state_flags(Some(target_tun), None)?;
                        let _ = self.restart_verge_ui_for_display_sync(global);
                        actions.push("restore clash tun".to_string());
                        return Ok(actions);
                    }
                    Err(error) => {
                        last_error = error.to_user_message();
                    }
                }
            }

            return Err(AppError::Network(format!(
                "恢复 Clash 状态失败: {last_error}"
            )));
        }

        Ok(actions)
    }

    fn start_clash_if_needed(&self, global: &GlobalSettings) -> AppResult<()> {
        let mut system = System::new_all();
        system.refresh_processes(ProcessesToUpdate::All, true);

        let running = system.processes().values().any(|proc| {
            let name = proc.name().to_string_lossy().to_ascii_lowercase();
            name.contains("clash") && name.contains("verge")
        });

        if running {
            return Ok(());
        }

        if global.clash_path.trim().is_empty() {
            return Ok(());
        }

        self.start_clash_hidden(&global.clash_path)
    }

    fn start_clash_hidden(&self, path: &str) -> AppResult<()> {
        windows_process::start_hidden_detached(
            path,
            &["--hidden".to_string(), "--silent".to_string()],
        )
    }

    fn restart_verge_ui_for_display_sync(&self, global: &GlobalSettings) -> AppResult<()> {
        let mut system = System::new_all();
        system.refresh_processes(ProcessesToUpdate::All, true);

        let mut ui_pids = Vec::new();
        let mut clash_path = if global.clash_path.trim().is_empty() {
            None
        } else {
            Some(global.clash_path.clone())
        };

        for (pid, process) in system.processes() {
            let name = process.name().to_string_lossy().to_ascii_lowercase();
            if name == "clash-verge.exe"
                || (name.contains("clash-verge") && !name.contains("service"))
            {
                ui_pids.push(*pid);
                if clash_path.is_none() {
                    if let Some(exe) = process.exe() {
                        clash_path = Some(exe.to_string_lossy().to_string());
                    }
                }
            }
        }

        if ui_pids.is_empty() {
            return Ok(());
        }

        for pid in ui_pids {
            if let Some(process) = system.process(pid) {
                let _ = process
                    .kill_with(Signal::Term)
                    .unwrap_or_else(|| process.kill());
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(600));

        if let Some(path) = clash_path {
            if Path::new(&path).exists() {
                self.start_clash_hidden(&path)?;
            }
        }

        Ok(())
    }

    fn sync_verge_state_flags(
        &self,
        tun_enabled: Option<bool>,
        system_proxy_enabled: Option<bool>,
    ) -> AppResult<()> {
        if let Some(value) = tun_enabled {
            self.set_verge_yaml_flag("enable_tun_mode", value)?;
        }
        if let Some(value) = system_proxy_enabled {
            self.set_verge_yaml_flag("enable_system_proxy", value)?;
        }
        Ok(())
    }

    fn set_verge_yaml_flag(&self, key: &str, value: bool) -> AppResult<()> {
        let flag = if value { "true" } else { "false" };

        for path in self.verge_yaml_candidates() {
            let content = fs::read_to_string(&path)
                .map_err(|error| AppError::Io(format!("读取 {} 失败: {error}", path.display())))?;

            let mut found = false;
            let mut lines = Vec::new();
            for line in content.lines() {
                if line.trim_start().starts_with(&format!("{key}:")) {
                    lines.push(format!("{key}: {flag}"));
                    found = true;
                } else {
                    lines.push(line.to_string());
                }
            }

            if !found {
                lines.push(format!("{key}: {flag}"));
            }

            let updated = format!("{}\n", lines.join("\n"));
            if updated != content {
                fs::write(&path, updated).map_err(|error| {
                    AppError::Io(format!("写入 {} 失败: {error}", path.display()))
                })?;
            }
        }

        Ok(())
    }

    fn verge_yaml_candidates(&self) -> Vec<PathBuf> {
        let Some(appdata) = std::env::var("APPDATA").ok() else {
            return Vec::new();
        };

        let roots = [
            "io.github.clash-verge-rev.clash-verge-rev",
            "io.github.clash-verge.clash-verge",
            "io.github.clash-verge.clash-verge-rev",
        ];

        roots
            .iter()
            .map(|root| Path::new(&appdata).join(root).join("verge.yaml"))
            .filter(|path| path.exists())
            .collect()
    }
}
