//! Names the Windows console window that shows the backend log.

#[cfg(windows)]
pub fn set(title: &str) {
    use windows_sys::Win32::System::Console::SetConsoleTitleW;

    let wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: the pointer is a valid NUL-terminated UTF-16 string that outlives the call.
    // The call fails without harm when the process has no console.
    unsafe { SetConsoleTitleW(wide.as_ptr()) };
}

#[cfg(not(windows))]
pub fn set(_title: &str) {}
