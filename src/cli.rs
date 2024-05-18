use clap::Parser;
use derive_more::From;
use serde::Serialize;

use crate::install::{self, Browser, NativeManifestJson};

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
    /// Install to browsers, separate by comma
    #[arg(short, long, use_value_delimiter = true, value_name = "BROWSERS")]
    install: Vec<Browser>,
}

pub fn main_cli() -> Result<(), &'static str> {
    let current_exe_path =
        std::env::current_exe().map_err(|_| "Failed to get current executable path")?;
    let args = Opts::parse();

    // Do installation
    if !args.install.is_empty() {
        let native_manifest_json = NativeManifestJson {
            path: current_exe_path,
            name: "FBrowserHelper".into(),
            description: "FBrowserHelper".into(),
            type_: "stdio".into(),
            allowed_origins: vec!["chrome-extension://*".into()],
            allowed_extensions: vec!["".into()],
        };

        // Install for each browser
        for browser in args.install {
            println!("Installing for {:?}", browser);
            install::install(browser, &native_manifest_json)?;
        }
    }

    Ok(())
}
