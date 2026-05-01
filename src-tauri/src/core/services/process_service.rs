use std::collections::HashSet;

use sysinfo::{ProcessesToUpdate, Signal, System};

use crate::core::domain::error::AppResult;
use crate::core::domain::types::{AppEntry, ProcessInfo};
use crate::infra::windows::process as windows_process;

#[derive(Default)]
pub struct ProcessService;

impl ProcessService {
    pub fn new() -> Self {
        Self
    }

    pub fn kill_process(&self, name: &str) -> AppResult<bool> {
        let mut system = System::new_all();
        system.refresh_processes(ProcessesToUpdate::All, true);

        let mut killed_any = false;
        for process in system.processes().values() {
            let process_name = process.name().to_string_lossy();
            if process_name.eq_ignore_ascii_case(name) {
                let killed = process
                    .kill_with(Signal::Kill)
                    .unwrap_or_else(|| process.kill());
                killed_any |= killed;
            }
        }

        Ok(killed_any)
    }

    pub fn start_process(&self, path: &str, args: &[String]) -> AppResult<()> {
        windows_process::start_hidden_detached(path, args)
    }

    pub fn close_all(&self, apps: &[AppEntry]) -> AppResult<()> {
        for app in apps {
            let _ = self.kill_process(&app.name)?;
        }
        Ok(())
    }

    pub fn start_all(&self, apps: &[AppEntry]) -> AppResult<()> {
        for app in apps {
            self.start_process(&app.path, &app.start_args)?;
        }
        Ok(())
    }

    pub fn start_all_missing(&self, apps: &[AppEntry]) -> AppResult<()> {
        let mut system = System::new_all();
        system.refresh_processes(ProcessesToUpdate::All, true);

        let mut running_names = HashSet::new();
        let mut running_paths = HashSet::new();

        for process in system.processes().values() {
            running_names.insert(process.name().to_string_lossy().to_ascii_lowercase());
            if let Some(exe) = process.exe() {
                running_paths.insert(exe.to_string_lossy().to_ascii_lowercase());
            }
        }

        for app in apps {
            let name_key = app.name.to_ascii_lowercase();
            let path_key = app.path.to_ascii_lowercase();
            let already_running = running_names.contains(&name_key)
                || (!path_key.is_empty() && running_paths.contains(&path_key));

            if already_running {
                continue;
            }

            self.start_process(&app.path, &app.start_args)?;
            running_names.insert(name_key);
            if !path_key.is_empty() {
                running_paths.insert(path_key);
            }
        }

        Ok(())
    }

    pub fn list_running_processes(&self) -> AppResult<Vec<ProcessInfo>> {
        let mut system = System::new_all();
        system.refresh_processes(ProcessesToUpdate::All, true);

        let mut set = HashSet::new();
        let mut list = Vec::new();

        for process in system.processes().values() {
            let Some(exe) = process.exe() else {
                continue;
            };
            let path = exe.to_string_lossy().to_string();
            if path.is_empty() {
                continue;
            }

            if path.to_ascii_lowercase().contains("c:\\windows") {
                continue;
            }

            if !set.insert(path.to_ascii_lowercase()) {
                continue;
            }

            list.push(ProcessInfo {
                name: process.name().to_string_lossy().to_string(),
                path,
            });
        }

        list.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
        Ok(list)
    }
}
