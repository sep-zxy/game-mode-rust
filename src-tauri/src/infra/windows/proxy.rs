use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
use winreg::RegKey;

use crate::core::domain::error::{AppError, AppResult};
use crate::core::domain::types::RuntimeState;

const INTERNET_SETTINGS_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Internet Settings";

pub fn disable_proxy(runtime: &mut RuntimeState) -> AppResult<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (_created_key, _) = hkcu
        .create_subkey(INTERNET_SETTINGS_PATH)
        .map_err(|error| AppError::System(format!("打开代理注册表失败: {error}")))?;

    let key = hkcu
        .open_subkey_with_flags(INTERNET_SETTINGS_PATH, KEY_READ | KEY_WRITE)
        .map_err(|error| AppError::System(format!("打开代理注册表失败: {error}")))?;

    let proxy_enable: u32 = key.get_value("ProxyEnable").unwrap_or(0);
    let proxy_server: String = key.get_value("ProxyServer").unwrap_or_default();

    runtime.windows_proxy_enable = Some(proxy_enable);
    runtime.windows_proxy_server = Some(proxy_server);

    key.set_value("ProxyEnable", &0u32)
        .map_err(|error| AppError::Permission(format!("写入 ProxyEnable 失败: {error}")))?;

    Ok(())
}

pub fn restore_proxy(runtime: &RuntimeState) -> AppResult<()> {
    if runtime.windows_proxy_enable.is_none() && runtime.windows_proxy_server.is_none() {
        return Ok(());
    }

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey_with_flags(INTERNET_SETTINGS_PATH, KEY_READ | KEY_WRITE)
        .map_err(|error| AppError::System(format!("打开代理注册表失败: {error}")))?;

    if let Some(enable) = runtime.windows_proxy_enable {
        key.set_value("ProxyEnable", &enable)
            .map_err(|error| AppError::Permission(format!("恢复 ProxyEnable 失败: {error}")))?;
    }

    if let Some(server) = &runtime.windows_proxy_server {
        key.set_value("ProxyServer", server)
            .map_err(|error| AppError::Permission(format!("恢复 ProxyServer 失败: {error}")))?;
    }

    Ok(())
}
