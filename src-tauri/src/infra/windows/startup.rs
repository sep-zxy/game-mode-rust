use winreg::enums::{HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SZ};
use winreg::RegKey;

use crate::core::domain::error::{AppError, AppResult};

const RUN_KEY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const RUN_VALUE_NAME: &str = "GameModeSwitcherRust";
const STARTUP_ARGS: &str = "--startup-silent --no-elevate";

pub fn is_enabled() -> AppResult<bool> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey_with_flags(RUN_KEY_PATH, KEY_QUERY_VALUE)
        .map_err(|_| AppError::System("读取开机自启键失败".to_string()))?;

    let value: Result<String, _> = key.get_value(RUN_VALUE_NAME);
    Ok(value.map(|text| !text.trim().is_empty()).unwrap_or(false))
}

pub fn set_enabled(enabled: bool) -> AppResult<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(RUN_KEY_PATH)
        .map_err(|error| AppError::System(format!("创建开机自启键失败: {error}")))?;

    if enabled {
        let command = build_startup_command()?;
        key.set_value(RUN_VALUE_NAME, &command)
            .map_err(|error| AppError::System(format!("写入开机自启失败: {error}")))?;
    } else if key.get_raw_value(RUN_VALUE_NAME).is_ok() {
        key.delete_value(RUN_VALUE_NAME)
            .map_err(|error| AppError::System(format!("删除开机自启失败: {error}")))?;
    }

    let _ = REG_SZ;
    let _ = KEY_SET_VALUE;
    Ok(())
}

fn build_startup_command() -> AppResult<String> {
    let exe = std::env::current_exe()
        .map_err(|error| AppError::System(format!("读取当前可执行路径失败: {error}")))?;
    Ok(format!("\"{}\" {}", exe.to_string_lossy(), STARTUP_ARGS))
}
