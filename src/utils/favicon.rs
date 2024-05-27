use std::io::Read;

use ico::IconImage;
use url::Url;

use crate::log;

#[allow(dead_code)]
#[derive(Debug)]
pub enum GetFaviconError {
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

/// Get the favicon from a URL
///
/// Uses only the domain part and queries the icon from Google
pub fn get_favicon_from_url(url: &Url) -> Result<String, GetFaviconError> {
    let domain = url.domain().ok_or(GetFaviconError::UrlDomainError)?;
    let url_without_path = format!("{}://{}", url.scheme(), domain);

    let icon_file = format!("{}.ico", domain);

    // Check if the icon file already exists
    if std::fs::metadata(&icon_file).is_ok() {
        return Ok(icon_file);
    }

    // Fetch the icon and convert to ico before saving
    let icon = reqwest::blocking::get(format!("https://t2.gstatic.com/faviconV2?client=SOCIAL&type=FAVICON&fallback_opts=TYPE,SIZE,URL&url={}&size=128", url_without_path))?;

    let image = image::load_from_memory(icon.bytes()?.to_vec().as_slice()).unwrap();
    let width = image.width();
    let height = image.height();
    let bytevector = image.to_rgba8().into_vec();

    let icondata = IconImage::from_rgba_data(width as u32, height as u32, bytevector);

    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);
    icon_dir.add_entry(ico::IconDirEntry::encode(&icondata)?);
    icon_dir.write(std::fs::File::create(&icon_file)?)?;
    Ok(icon_file)
}
