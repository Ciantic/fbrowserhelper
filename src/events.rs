use std::panic;

use serde::{Deserialize, Serialize};
use windows::Win32::Foundation::HWND;

use crate::log;
use crate::utils::favicon::get_favicon_from_url;
use crate::utils::native_messaging::{read_message, send_message};
use crate::utils::win32::{
    allow_maximize_and_snapping, get_active_window, get_process_name, get_window_class,
    get_window_title, set_icon, ungroup_taskbar_button,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum MessageFromBrowser {
    GetActiveWindow,
    UngroupTaskbarButton { hwnd: u32, new_id: String },
    SetTaskbarIcon { hwnd: u32, icon_url: String },
    Quit,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum MessageToBrowser {
    ActiveWindow {
        hwnd: u32,
        class_name: String,
        title: String,
        process_name: String,
    },
    Ok,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum MessageToError {
    UrlParsingError {
        message: String,
    },
    Error {
        message: String,
    },
    IoError {
        kind: String,
        message: String,
    },
    JsonParseError {
        message: String,
    },
    Panic {
        message: String,
        file: Option<String>,
        line: Option<u32>,
    },
    Quit,
}

fn event_handler(msg: MessageFromBrowser) -> Result<MessageToBrowser, MessageToError> {
    match msg {
        MessageFromBrowser::GetActiveWindow => {
            let nhwnd = get_active_window();
            let class_name = get_window_class(nhwnd);
            let process_name = get_process_name(nhwnd);
            let title = get_window_title(nhwnd);
            let hwnd = nhwnd.0 as u32;

            Ok(MessageToBrowser::ActiveWindow {
                hwnd,
                class_name,
                process_name,
                title,
            })
        }

        MessageFromBrowser::UngroupTaskbarButton { hwnd, new_id } => {
            ungroup_taskbar_button(HWND(hwnd as isize), &new_id);
            Ok(MessageToBrowser::Ok)
        }

        MessageFromBrowser::SetTaskbarIcon { hwnd, icon_url } => {
            log(&format!("Setting icon for hwnd {} to {}", hwnd, icon_url));
            let hwnd = HWND(hwnd as isize);

            let url = url::Url::parse(&icon_url).map_err(|_| MessageToError::UrlParsingError {
                message: "Invalid favicon URL".into(),
            })?;

            let favicon_path = get_favicon_from_url(&url).map_err(|err| MessageToError::Error {
                message: "Failed to get favicon".into(),
                // message: format!("{:?}", err),
            })?;

            set_icon(hwnd, favicon_path);

            // When using popups in Firefox, the window is not maximizable, enable that
            allow_maximize_and_snapping(hwnd);

            log(&format!("Icon set for hwnd {}", hwnd.0));

            Ok(MessageToBrowser::Ok)
        }

        MessageFromBrowser::Quit => Err(MessageToError::Quit),
    }
}

pub fn main_event_loop() -> Result<(), MessageToError> {
    // Send panic messages to the browser
    panic::set_hook(Box::new(|info: &std::panic::PanicInfo| {
        let response = MessageToError::Panic {
            message: format!("{}", info),
            file: info.location().map(|l| l.file().to_string()),
            line: info.location().map(|l| l.line()),
        };
        log(&format!("Panic: {:?}", response));
        let _ = send_message(std::io::stdout(), &response);
    }));

    // Event loop
    loop {
        // Read message error ends the loop
        let msg = read_message(std::io::stdin())?;

        // Event handler error does not end the loop, except Quit
        match event_handler(msg) {
            Ok(msg) => {
                send_message(std::io::stdout(), &msg).unwrap();
            }
            Err(msg) => {
                if let MessageToError::Quit = msg {
                    break;
                }
                send_message(std::io::stdout(), &msg).unwrap();
            }
        }
    }
    Ok(())
}
