use windows_sys::Win32::Networking::WinInet::InternetSetOptionW;

use crate::core::domain::error::{AppError, AppResult};

const INTERNET_OPTION_SETTINGS_CHANGED: u32 = 39;
const INTERNET_OPTION_REFRESH: u32 = 37;

pub fn refresh_internet_options() -> AppResult<()> {
    let changed = unsafe {
        InternetSetOptionW(
            std::ptr::null(),
            INTERNET_OPTION_SETTINGS_CHANGED,
            std::ptr::null(),
            0,
        )
    };
    let refresh = unsafe {
        InternetSetOptionW(
            std::ptr::null(),
            INTERNET_OPTION_REFRESH,
            std::ptr::null(),
            0,
        )
    };

    if changed == 0 || refresh == 0 {
        return Err(AppError::System(
            "刷新 WinINet 代理状态失败，请检查系统权限".to_string(),
        ));
    }

    Ok(())
}
