use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, clap::ValueEnum, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Browser {
    Chrome,
    Firefox,
    Edge,
}

// https://developer.chrome.com/docs/extensions/develop/concepts/native-messaging#native-messaging-host
#[derive(Serialize)]
pub struct NativeManifestJson {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
    #[serde(rename = "type")]
    pub type_: String,
    pub allowed_origins: Vec<String>,
    pub allowed_extensions: Vec<String>,
}

pub fn install(browser: Browser, extension: &NativeManifestJson) -> Result<(), &'static str> {
    let manifest_json_path = extension.path.with_file_name("native_manifest.json");

    // TODO: executable path could be relative in the manifest.json

    // Write the manifest.json
    std::fs::write(
        &manifest_json_path,
        serde_json::to_string_pretty(&extension).map_err(|_| "Failed to serialize JSON")?,
    )
    .map_err(|_| "Failed to write manifest.json")?;

    // Create the registry key, point it to the manifest.json file
    winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
        .create_subkey(
            PathBuf::from(match browser {
                Browser::Chrome => r"Software\Google\Chrome\NativeMessagingHosts",
                Browser::Firefox => r"Software\Mozilla\NativeMessagingHosts",
                Browser::Edge => r"Software\Microsoft\Edge\NativeMessagingHosts",
            })
            .join(&extension.name),
        )
        .map_err(|_| "Failed to create registry key")?
        .0
        .set_value("", &manifest_json_path.as_os_str())
        .map_err(|_| "Failed to set registry key value")?;

    Ok(())
}
