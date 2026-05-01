use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::{CloseHandle, GetLastError};
use windows_sys::Win32::System::Threading::{
    CreateProcessW, CREATE_NEW_PROCESS_GROUP, DETACHED_PROCESS, PROCESS_INFORMATION,
    STARTF_USESHOWWINDOW, STARTUPINFOW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::SW_HIDE;

use crate::core::domain::error::{AppError, AppResult};

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn start_hidden_detached(path: &str, args: &[String]) -> AppResult<()> {
    if !Path::new(path).exists() {
        return Err(AppError::NotFound(format!("可执行文件不存在: {path}")));
    }

    let command_line = build_command_line(path, args);
    let mut command_line_wide = wide_null(&command_line);
    let application_wide = wide_null(path);
    let working_directory_wide = Path::new(path)
        .parent()
        .and_then(|parent| parent.to_str())
        .map(wide_null);

    let mut startup_info: STARTUPINFOW = unsafe { std::mem::zeroed() };
    startup_info.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
    startup_info.dwFlags = STARTF_USESHOWWINDOW;
    startup_info.wShowWindow = SW_HIDE as u16;

    let mut process_info: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
    let creation_flags = DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW;

    let ok = unsafe {
        CreateProcessW(
            application_wide.as_ptr(),
            command_line_wide.as_mut_ptr(),
            null(),
            null(),
            0,
            creation_flags,
            null(),
            working_directory_wide
                .as_ref()
                .map_or(null(), |value| value.as_ptr()),
            &startup_info,
            &mut process_info,
        )
    };

    if ok == 0 {
        let last_error = unsafe { GetLastError() };
        return Err(AppError::Process(format!(
            "启动进程失败，错误码: {last_error}, 路径: {path}"
        )));
    }

    unsafe {
        if process_info.hThread != null_mut() {
            CloseHandle(process_info.hThread);
        }
        if process_info.hProcess != null_mut() {
            CloseHandle(process_info.hProcess);
        }
    }

    Ok(())
}

fn wide_null(value: &str) -> Vec<u16> {
    OsStr::new(value)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn build_command_line(path: &str, args: &[String]) -> String {
    let mut parts = Vec::with_capacity(args.len() + 1);
    parts.push(quote_windows_arg(path));
    for arg in args {
        parts.push(quote_windows_arg(arg));
    }
    parts.join(" ")
}

fn quote_windows_arg(arg: &str) -> String {
    if arg.is_empty() {
        return "\"\"".to_string();
    }

    let need_quote = arg.chars().any(|ch| ch.is_whitespace() || ch == '"');
    if !need_quote {
        return arg.to_string();
    }

    let mut result = String::with_capacity(arg.len() + 2);
    result.push('"');

    let mut backslashes = 0usize;
    for ch in arg.chars() {
        match ch {
            '\\' => {
                backslashes += 1;
            }
            '"' => {
                result.push_str(&"\\".repeat(backslashes * 2 + 1));
                result.push('"');
                backslashes = 0;
            }
            _ => {
                if backslashes > 0 {
                    result.push_str(&"\\".repeat(backslashes));
                    backslashes = 0;
                }
                result.push(ch);
            }
        }
    }

    if backslashes > 0 {
        result.push_str(&"\\".repeat(backslashes * 2));
    }

    result.push('"');
    result
}
