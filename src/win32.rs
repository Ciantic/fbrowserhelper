use ico::IconImage;
use url::Url;
use windows::{
    core::{s, HSTRING, PCWSTR},
    Win32::{
        Foundation::*,
        Graphics::Gdi::ValidateRect,
        Storage::EnhancedStorage::{PKEY_AppUserModel_ID, PKEY_AppUserModel_PreventPinning},
        System::{
            Com::StructuredStorage::{
                InitPropVariantFromBooleanVector, InitPropVariantFromStringVector,
            },
            LibraryLoader::GetModuleHandleA,
        },
        UI::{
            Shell::PropertiesSystem::{IPropertyStore, SHGetPropertyStoreForWindow},
            WindowsAndMessaging::*,
        },
    },
};

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
                                set_maximize(target_window);

                                // Set the icon
                                let icon_path = get_favicon_from_url(&url);
                                match icon_path {
                                    Err(err) => {
                                        println!("Error {:?}", err);
                                    }
                                    Ok(icon_path) => set_icon(target_window, icon_path),
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

fn set_maximize(window: HWND) {
    let style = unsafe { GetWindowLongA(window, GWL_STYLE) };
    unsafe { SetWindowLongA(window, GWL_STYLE, style | WS_MAXIMIZEBOX.0 as i32) };
}

fn set_icon(window: HWND, icon_path: String) {
    let icon_path_hstring = HSTRING::from(&icon_path);
    let icon_path_pcstr = PCWSTR(icon_path_hstring.as_ptr());
    let hicon =
        unsafe { LoadImageW(None, icon_path_pcstr, IMAGE_ICON, 0, 0, LR_LOADFROMFILE).unwrap() };
    println!("Set icon {} {:?}", &icon_path, hicon.0);
    let _ = unsafe {
        SendMessageA(
            window,
            WM_SETICON,
            WPARAM(ICON_SMALL as usize),
            LPARAM(hicon.0 as isize),
        )
    };
}

fn get_window_title(window: HWND) -> String {
    let mut title = [0u16; 256];
    let _len = unsafe { GetWindowTextW(window, &mut title) };
    let strr = HSTRING::from_wide(&title.split_at(_len as usize).0).unwrap_or_default();
    strr.to_string()
}

fn get_window_class(window: HWND) -> String {
    let mut class_name = [0u16; 256];
    let _len = unsafe { RealGetWindowClassW(window, &mut class_name) };
    let strr = HSTRING::from_wide(&class_name.split_at(_len as usize).0).unwrap_or_default();
    strr.to_string()
}

fn ungroup_taskbar_button(window: HWND, new_id: &str) {
    unsafe {
        let store: IPropertyStore = SHGetPropertyStoreForWindow(window).unwrap();

        // Ungroup taskbar button
        let new_id_hstr = HSTRING::from(new_id);
        let new_id_pcwstr = PCWSTR(new_id_hstr.as_ptr());
        let prop_variant = InitPropVariantFromStringVector(Some(&[new_id_pcwstr])).unwrap();
        store
            .SetValue(&PKEY_AppUserModel_ID, &prop_variant)
            .unwrap();

        // Prevent pinning (it says this should be done *before* ungrouping, but it worked after too \_o_/)
        let variant = InitPropVariantFromBooleanVector(Some(&[BOOL(1)])).unwrap();
        store
            .SetValue(&PKEY_AppUserModel_PreventPinning, &variant)
            .unwrap();
    }
}

fn get_url_from_string(string: &str) -> Option<Url> {
    let url = Url::parse(string.split(" ").next().unwrap());
    match url {
        Ok(url) => Some(url),
        Err(_) => None,
    }
}

#[allow(dead_code)]
#[derive(Debug)]
enum GetFaviconError {
    UrlDomainError,
    NotInPngFormatError,
    ReqwestError(reqwest::Error),
    IOError(std::io::Error),
    LodepngError(lodepng::Error),
}

// Allow IOError to be converted to GetFaviconError
impl From<std::io::Error> for GetFaviconError {
    fn from(error: std::io::Error) -> Self {
        GetFaviconError::IOError(error)
    }
}

// Allow ReqwestError to be converted to GetFaviconError
impl From<reqwest::Error> for GetFaviconError {
    fn from(error: reqwest::Error) -> Self {
        GetFaviconError::ReqwestError(error)
    }
}

// Allow LodepngError to be converted to GetFaviconError
impl From<lodepng::Error> for GetFaviconError {
    fn from(error: lodepng::Error) -> Self {
        GetFaviconError::LodepngError(error)
    }
}

fn get_favicon_from_url(url: &Url) -> Result<String, GetFaviconError> {
    let icon_file = format!(
        "{}.ico",
        url.domain().ok_or(GetFaviconError::UrlDomainError)?
    );

    // Check if the icon file already exists
    if std::fs::metadata(&icon_file).is_ok() {
        return Ok(icon_file);
    }

    // Fetch the icon and convert to ico before saving
    let icon = reqwest::blocking::get(format!("https://t2.gstatic.com/faviconV2?client=SOCIAL&type=FAVICON&fallback_opts=TYPE,SIZE,URL&url={}&size=128", url))?;

    if icon.headers().get("content-type").unwrap() != "image/png" {
        // This should not happen, as Google's favicon service always returns a PNG
        println!(
            "Format is {:?}",
            icon.headers().get("content-type").unwrap()
        );
        return Err(GetFaviconError::NotInPngFormatError);
    }

    let pngbytes = icon.bytes()?.to_vec();
    let decoded_png = lodepng::decode32(pngbytes.as_slice())?;
    let bytevector: Vec<u8> = decoded_png
        .buffer
        .iter()
        .flat_map(|pixel| [pixel.r, pixel.g, pixel.b, pixel.a])
        .collect();
    let icondata = IconImage::from_rgba_data(
        decoded_png.width as u32,
        decoded_png.height as u32,
        bytevector,
    );

    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);
    icon_dir.add_entry(ico::IconDirEntry::encode(&icondata)?);
    icon_dir.write(std::fs::File::create(&icon_file)?)?;
    Ok(icon_file)
}

// TODO: Pinning relaunch support:
// PKEY_AppUserModel_RelaunchCommand to define a relaunch command when pinned
// PKEY_AppUserModel_RelaunchDisplayNameResource name of the pinned app
// PKEY_AppUserModel_RelaunchIconResource Pinned icon resource

// https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-preventpinning
// https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-relaunchcommand
// https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-relaunchdisplaynameresource
