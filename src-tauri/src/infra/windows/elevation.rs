use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;

use windows_sys::Win32::UI::Shell::{IsUserAnAdmin, ShellExecuteW};
use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

use crate::core::domain::error::{AppError, AppResult};

pub fn is_admin() -> bool {
    unsafe { IsUserAnAdmin() != 0 }
}

pub fn relaunch_as_admin(args: &[String]) -> AppResult<()> {
    let exe = std::env::current_exe()
        .map_err(|error| AppError::System(format!("读取当前可执行路径失败: {error}")))?;

    let params = args
        .iter()
        .skip(1)
        .map(|arg| quote_arg(arg))
        .collect::<Vec<_>>()
        .join(" ");

    let op = to_wide("runas");
    let exe_w = to_wide(exe.to_string_lossy().as_ref());
    let params_w = to_wide(&params);

    let result = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            op.as_ptr(),
            exe_w.as_ptr(),
            if params.is_empty() {
                std::ptr::null()
            } else {
                params_w.as_ptr()
            },
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    };

    if result as isize <= 32 {
        return Err(AppError::Permission(
            "请求管理员权限失败，请手动右键以管理员身份运行".to_string(),
        ));
    }

    Ok(())
}

fn quote_arg(value: &str) -> String {
    if value.contains(' ') {
        format!("\"{value}\"")
    } else {
        value.to_string()
    }
}

fn to_wide(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}
