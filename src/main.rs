use clap::Parser;

mod utils;
use events::main_event_loop;
use utils::native_manifest_installer::{install, Browser, NativeManifestJson};
mod events;
pub(crate) use utils::log::log;

// Clap intro
//
// Derive tutorial
// https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html
//
// Parameters for #[arg] are methods in:
// https://docs.rs/clap/latest/clap/struct.Arg.html
//
// Parameters for #[command] are methods in:
// https://docs.rs/clap/latest/clap/struct.Command.html

/// FBrowserHelper
#[derive(Parser, Debug)]
#[command(version, about, long_about = "None", arg_required_else_help = true)]
struct Opts {
    /// Extension calling the app
    #[arg(index = 1, default_value = None)]
    extension: Option<String>,

    /// Manifest in use (sent only by Firefox)
    #[arg(index = 2, default_value = None)]
    extension_manifest: Option<String>,

    /// Parent window ID (sent only by Chrome)
    #[arg(long, default_value = None)]
    parent_window: Option<String>,

    /// Install to browsers, separate by comma
    #[arg(short, long, use_value_delimiter = true, value_name = "BROWSERS")]
    install: Vec<Browser>,
}

pub fn main() -> Result<(), &'static str> {
    let args = Opts::parse();
    log(&format!("Starting... {:?}", args));

    // Get current executable path
    let current_exe_path =
        std::env::current_exe().map_err(|_| "Failed to get current executable path")?;

    // If extension is provided, run event loop
    if args.extension.is_some() {
        main_event_loop();
    }

    // Do installation
    if !args.install.is_empty() {
        let native_manifest_json = NativeManifestJson {
            path: current_exe_path,
            name: "f_browser_helper_app".into(),
            description: "Browser helper app".into(),
            type_: "stdio".into(),
            allowed_origins: vec!["chrome-extension://dnmkkgomoldfnbpjolhekmnoligmhdnc/".into()],
            allowed_extensions: vec!["f_browser_helper_ext@oksidi.com".into()],
        };

        // Install for each browser
        for browser in args.install {
            println!("Installing for {:?}", browser);
            install(browser, &native_manifest_json)?;
        }
    }

    log("Quitting...");

    Ok(())
}
