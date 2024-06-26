use std::thread;

use ico::IconImage;
use url::Url;
use windows::{
    core::{s, HSTRING, PCWSTR, PROPVARIANT, PWSTR},
    Win32::{
        Foundation::*,
        Graphics::Gdi::ValidateRect,
        Storage::EnhancedStorage::{
            PKEY_AppUserModel_ID, PKEY_AppUserModel_PreventPinning,
            PKEY_AppUserModel_RelaunchIconResource,
        },
        System::{
            Com::StructuredStorage::{
                InitPropVariantFromBooleanVector, InitPropVariantFromStringVector,
            },
            LibraryLoader::{GetModuleFileNameW, GetModuleHandleA},
            ProcessStatus::EnumProcessModules,
            Threading::{
                OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
                PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ,
            },
        },
        UI::{
            Shell::PropertiesSystem::{IPropertyStore, SHGetPropertyStoreForWindow},
            WindowsAndMessaging::*,
        },
    },
};

use crate::{log, utils::favicon::get_favicon_from_url};

fn main() -> windows::core::Result<()> {
    unsafe {
        let instance = GetModuleHandleA(None)?;
        debug_assert!(instance.0 != 0);

        let window_class = s!("window");

        let wc = WNDCLASSA {
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hInstance: instance.into(),
            lpszClassName: window_class,

            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            ..Default::default()
        };

        let atom = RegisterClassA(&wc);
        debug_assert!(atom != 0);

        CreateWindowExA(
            WINDOW_EX_STYLE::default(),
            window_class,
            s!("This is a sample window"),
            // WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            instance,
            None,
        );

        let mut message = MSG::default();

        while GetMessageA(&mut message, None, 0, 0).into() {
            DispatchMessageA(&message);
        }

        Ok(())
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    static mut SHELLHOOK_MSG: u32 = 0;
    unsafe {
        match message {
            m if m == SHELLHOOK_MSG => {
                if wparam.0 == HSHELL_WINDOWCREATED as usize {
                    let target_window = HWND(lparam.0);
                    let class_name = get_window_class(target_window);
                    let name = get_window_title(target_window);
                    // println!("Create window {} {}", class_name, name);
                    if "MozillaDialogClass" == class_name {
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            let name = get_window_title(target_window);
                            println!("Title {}", name);
                            if let Some(url) = get_url_from_string(&name) {
                                println!("URL {}", url);
                                // Ungroups (but groups with the URL)
                                ungroup_taskbar_button(target_window, &url.to_string());

                                // Allow maximizing and snappin the window
                                allow_maximize_and_snapping(target_window);

                                // Set the icon
                                match get_favicon_from_url(&url) {
                                    Err(err) => {
                                        println!("Error {:?}", err);
                                    }
                                    Ok(icon_path) => {
                                        set_icon(target_window, &icon_path);
                                        set_pinned_taskbar_icon(window, &icon_path);
                                    }
                                }
                            }
                        });
                    }
                } else if wparam.0 == HSHELL_WINDOWDESTROYED as usize {
                    // println!("Window destroyed {}", lparam.0.to_string().as_str());
                }
                LRESULT(0)
            }
            WM_CREATE => {
                // println!("WM_CREATE");
                let _ = RegisterShellHookWindow(window);
                SHELLHOOK_MSG = RegisterWindowMessageA(s!("SHELLHOOK"));

                LRESULT(0)
            }

            WM_PAINT => {
                // println!("WM_PAINT");
                _ = ValidateRect(window, None);
                LRESULT(0)
            }
            WM_DESTROY => {
                let _ = DeregisterShellHookWindow(window);
                // println!("WM_DESTROY");
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}

pub fn allow_maximize_and_snapping(window: HWND) {
    let style = unsafe { GetWindowLongA(window, GWL_STYLE) };
    unsafe { SetWindowLongA(window, GWL_STYLE, style | WS_MAXIMIZEBOX.0 as i32) };
}

pub fn set_icon(window: HWND, icon_path: &str) {
    let icon_path_hstring = HSTRING::from(icon_path);
    let icon_path_pcstr = PCWSTR(icon_path_hstring.as_ptr());
    let hicon =
        unsafe { LoadImageW(None, icon_path_pcstr, IMAGE_ICON, 64, 64, LR_LOADFROMFILE).unwrap() };
    let hicon2 = unsafe {
        LoadImageW(None, icon_path_pcstr, IMAGE_ICON, 128, 128, LR_LOADFROMFILE).unwrap()
    };

    if hicon.is_invalid() || hicon2.is_invalid() {
        log(&format!("Failed to load icon: {:?}", icon_path));
        return;
    }

    let _ = unsafe {
        PostMessageW(
            window,
            WM_SETICON,
            WPARAM(ICON_SMALL as usize),
            LPARAM(hicon.0 as isize),
        )
    };
    let _ = unsafe {
        PostMessageW(
            window,
            WM_SETICON,
            WPARAM(ICON_BIG as usize),
            LPARAM(hicon2.0 as isize),
        )
    };
}

pub fn get_active_window() -> HWND {
    unsafe { GetForegroundWindow() }
}

pub fn get_window_title(window: HWND) -> String {
    let mut title = [0u16; 256];
    let _len = unsafe { GetWindowTextW(window, &mut title) };
    let strr = HSTRING::from_wide(&title.split_at(_len as usize).0).unwrap_or_default();
    strr.to_string()
}

pub fn get_window_class(window: HWND) -> String {
    let mut class_name = [0u16; 256];
    let _len = unsafe { RealGetWindowClassW(window, &mut class_name) };
    let strr = HSTRING::from_wide(&class_name.split_at(_len as usize).0).unwrap_or_default();
    strr.to_string()
}

pub fn ungroup_taskbar_button(window: HWND, new_id: &str) {
    unsafe {
        let store: IPropertyStore = SHGetPropertyStoreForWindow(window).unwrap();

        // Ungroup taskbar button
        let new_id_hstr = HSTRING::from(new_id);
        let new_id_pcwstr = PCWSTR(new_id_hstr.as_ptr());
        let prop_variant = InitPropVariantFromStringVector(Some(&[new_id_pcwstr])).unwrap();
        store
            .SetValue(&PKEY_AppUserModel_ID, &prop_variant)
            .unwrap();
    }
}

pub fn prevent_pinning_taskbar_button(window: HWND) {
    unsafe {
        let store: IPropertyStore = SHGetPropertyStoreForWindow(window).unwrap();

        // Prevent pinning (it says this should be done *before* ungrouping, but it worked after too \_o_/)
        let variant = InitPropVariantFromBooleanVector(Some(&[BOOL(1)])).unwrap();
        store
            .SetValue(&PKEY_AppUserModel_PreventPinning, &variant)
            .unwrap();
    }
}

pub fn unprevent_pinning_taskbar_button(window: HWND) {
    unsafe {
        let store: IPropertyStore = SHGetPropertyStoreForWindow(window).unwrap();

        let variant = PROPVARIANT::default();
        store
            .SetValue(&PKEY_AppUserModel_PreventPinning, &variant)
            .unwrap();
    }
}

pub fn set_pinned_taskbar_icon(window: HWND, favicon_path: &str) {
    unsafe {
        let store: IPropertyStore = SHGetPropertyStoreForWindow(window).unwrap();

        // Ungroup taskbar button
        let new_id_hstr = HSTRING::from(favicon_path);
        let new_id_pcwstr = PCWSTR(new_id_hstr.as_ptr());
        let prop_variant = InitPropVariantFromStringVector(Some(&[new_id_pcwstr])).unwrap();
        store
            .SetValue(&PKEY_AppUserModel_RelaunchIconResource, &prop_variant)
            .unwrap();
    }
}

pub fn clear_pinned_taskbar_icon(window: HWND) {
    unsafe {
        let store: IPropertyStore = SHGetPropertyStoreForWindow(window).unwrap();

        // Ungroup taskbar button
        let prop_variant = PROPVARIANT::default();
        // println!("Is empty variant {}", prop_variant.is_empty());
        store
            .SetValue(&PKEY_AppUserModel_RelaunchIconResource, &prop_variant)
            .unwrap();
    }
}

pub fn get_process_name(window: HWND) -> String {
    unsafe {
        // Get the process ID
        let mut process_id = 0;
        GetWindowThreadProcessId(window, Some(&mut process_id as *mut u32));
        if process_id == 0 {
            log("Failed to get process ID");
            return "".to_string();
        }

        // Get the process handle
        let hproc =
            OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, process_id).unwrap_or_default();
        if hproc.0 == 0 {
            log("Failed to get process handle");
            return "".to_string();
        }

        // Get the process name
        let mut exebuffer = [0u16; 1024];
        let exepwstr = PWSTR::from_raw(&mut exebuffer as *mut u16);
        let mut exelen = 1024;
        if let Err(err) =
            QueryFullProcessImageNameW(hproc, PROCESS_NAME_FORMAT::default(), exepwstr, &mut exelen)
        {
            log(&format!("Failed to query process name: {:?}", err));
            return "".to_string();
        }
        exepwstr.to_string().unwrap_or_default()
    }
}

fn get_url_from_string(string: &str) -> Option<Url> {
    let url = Url::parse(string.split(" ").next().unwrap());
    match url {
        Ok(url) => Some(url),
        Err(_) => None,
    }
}

// TODO: Pinning relaunch support:
// PKEY_AppUserModel_RelaunchCommand to define a relaunch command when pinned
// PKEY_AppUserModel_RelaunchDisplayNameResource name of the pinned app
// PKEY_AppUserModel_RelaunchIconResource Pinned icon resource

// https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-preventpinning
// https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-relaunchcommand
// https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-relaunchdisplaynameresource

#[cfg(test)]
mod tests {
    use windows::core::w;

    use super::*;

    #[test]
    fn test_set_icon() {}
}
