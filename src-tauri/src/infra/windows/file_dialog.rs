use std::mem::size_of;

use windows_sys::Win32::UI::Controls::Dialogs::{
    CommDlgExtendedError, GetOpenFileNameW, OFN_EXPLORER, OFN_FILEMUSTEXIST, OFN_HIDEREADONLY,
    OFN_PATHMUSTEXIST, OPENFILENAMEW,
};

use crate::core::domain::error::{AppError, AppResult};

pub fn pick_executable_path() -> AppResult<Option<String>> {
    let mut file_buffer = [0u16; 32768];
    let filter: Vec<u16> = "Executable Files (*.exe)\0*.exe\0All Files (*.*)\0*.*\0\0"
        .encode_utf16()
        .collect();

    let mut ofn: OPENFILENAMEW = unsafe { std::mem::zeroed() };
    ofn.lStructSize = size_of::<OPENFILENAMEW>() as u32;
    ofn.lpstrFilter = filter.as_ptr();
    ofn.lpstrFile = file_buffer.as_mut_ptr();
    ofn.nMaxFile = file_buffer.len() as u32;
    ofn.Flags = OFN_EXPLORER | OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_HIDEREADONLY;

    let ok = unsafe { GetOpenFileNameW(&mut ofn as *mut OPENFILENAMEW) };
    if ok != 0 {
        let path_len = file_buffer
            .iter()
            .position(|item| *item == 0)
            .unwrap_or_default();
        let path = String::from_utf16_lossy(&file_buffer[..path_len]);
        if path.trim().is_empty() {
            return Ok(None);
        }
        return Ok(Some(path));
    }

    let extended_error = unsafe { CommDlgExtendedError() };
    if extended_error == 0 {
        return Ok(None);
    }

    Err(AppError::System(format!(
        "打开文件选择器失败，错误码: {extended_error}"
    )))
}
