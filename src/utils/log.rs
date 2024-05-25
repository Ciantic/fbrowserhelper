#[cfg(debug_assertions)]
extern "system" {
    fn OutputDebugStringW(lpOutputString: windows::core::PCWSTR);
}

/// Log to OutputDebugStringW
///
/// Use win32 executable DebugView to see the logs
#[cfg(debug_assertions)]
pub fn log(s: &str) {
    unsafe {
        let notepad = format!("FBrowserHelper: {}\0", s)
            .encode_utf16()
            .collect::<Vec<_>>();
        let pw = windows::core::PCWSTR::from_raw(notepad.as_ptr());
        OutputDebugStringW(pw);
    }
}

#[cfg(not(debug_assertions))]
#[inline]
pub fn log(_s: &str) {}
