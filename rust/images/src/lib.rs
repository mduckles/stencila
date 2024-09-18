use std::{
    fs::{copy, create_dir_all, read_to_string, File},
    hash::{Hash, Hasher},
    io::{Cursor, Write},
    path::{Path, PathBuf},
};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use common::{
    eyre::{bail, OptionExt, Result},
    itertools::Itertools,
    seahash::SeaHasher,
};
use image::{ImageFormat, ImageReader};
use mime_guess::from_path;

/**
 * Covert an image URL to a HTTP or data URI
 *
 * URL beginning with `http://`, `https://`, or `data:` will be returned unchanged.
 * Other URLs, including those beginning with `file://`, are assumed to be filesystem
 * path and will be converted to a sata URI.
 */
pub fn ensure_http_or_data_uri(url: &str) -> Result<String> {
    if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("data:") {
        return Ok(url.into());
    }

    let path = url.strip_prefix("file://").unwrap_or(url);
    let path = PathBuf::from(path);

    path_to_data_uri(&path)
}

/**
 * Convert a filesystem path to an image into a data URI
 */
pub fn path_to_data_uri(path: &Path) -> Result<String> {
    let mime_type = from_path(path).first_or_octet_stream();

    if mime_type.type_() != mime::IMAGE {
        bail!("Path is not an image: {}", path.display())
    }

    let encoded = if mime_type.subtype() == mime::SVG {
        // Plain text images
        STANDARD.encode(read_to_string(path)?)
    } else {
        // Binary images
        let image = ImageReader::open(path)?.decode()?;
        let mut bytes: Vec<u8> = Vec::new();
        image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)?;
        STANDARD.encode(&bytes)
    };

    Ok(format!("data:{};base64,{}", mime_type, encoded))
}

/**
 * Convert a data URI into an image file
 *
 * The image will be converted into an image file with a name
 * based on the hash of the URI and an extension based on the
 * type of data URI.
 *
 * # Arguments
 *
 * - `data_uri`: the data URI
 * - `dest_dir`: the destination directory
 *
 * # Returns
 *
 * The path of the generated file (including the `dest_dir`).
 */
pub fn data_uri_to_path(data_uri: &str, dest_dir: &Path) -> Result<PathBuf> {
    // Parse the data URI
    let Some((header, data)) = data_uri.split(',').collect_tuple() else {
        bail!("Invalid data URI format");
    };

    // Extract the MIME type
    let mime_type = header
        .split(';')
        .next()
        .and_then(|mime_type| mime_type.strip_prefix("data:"))
        .ok_or_eyre("Invalid data URI header")?;

    // Determine the file extension based on the MIME type
    let extension = match mime_type {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        _ => bail!("Unsupported image format: {}", mime_type),
    };

    // Decode the Base64 data
    let decoded_data = STANDARD.decode(data.as_bytes())?;

    // Generate a hash of the data URI
    let mut hash = SeaHasher::new();
    data_uri.hash(&mut hash);
    let hash = hash.finish();

    // Create the path to the file
    let dest_path = dest_dir.join(format!("{:x}.{}", hash, extension));

    // Write the decoded data to the file
    let mut file = File::create(&dest_path)?;
    file.write_all(&decoded_data)?;

    Ok(dest_path)
}

/**
 * Convert a file URI to a filesystem path to an image
 *
 * The absolute path of the source image will be resolved
 * from `file_uri` and `src_path` and the image copied to `dest_dir`.
 *
 * # Arguments
 *
 * - `file_uri`: an absolute or relative filesystem path, which may be prefixed with `file://`
 * - `src_path`: the path that any relative paths are relative to
 * - `dest_dir`: the destination directory
 *
 * # Returns
 *
 * The path of the generated file (including the `dest_dir`).
 */
pub fn file_uri_to_path(
    file_uri: &str,
    src_path: Option<&Path>,
    dest_dir: &Path,
) -> Result<PathBuf> {
    // Handle the file URI, stripping the "file://" prefix if present
    let path_str = file_uri.strip_prefix("file://").unwrap_or(file_uri);
    let path = PathBuf::from(path_str);

    // Resolve the src path
    let src_path = if path.is_absolute() {
        path
    } else {
        match src_path {
            Some(src) => {
                if src.is_dir() {
                    src.join(path)
                } else {
                    src.parent()
                        .map(|parent| parent.join(path))
                        .unwrap_or_else(|| src.to_path_buf())
                }
            }
            None => std::env::current_dir()?.join(path),
        }
    };

    // Ensure the source file exists
    if !src_path.exists() {
        bail!("Source file does not exist: {:?}", src_path);
    }

    // Generate a unique filename for the destination
    let mut hash = SeaHasher::new();
    src_path.hash(&mut hash);
    let hash = hash.finish();
    let ext = src_path
        .extension()
        .ok_or_eyre("Invalid source file name")?;
    let dest_path = dest_dir.join(format!("{:x}.{}", hash, ext.to_string_lossy()));

    // Ensure the destination directory exists
    if !dest_dir.exists() {
        create_dir_all(dest_dir)?;
    }

    // Copy the file to the destination directory
    copy(&src_path, &dest_path)?;

    Ok(dest_path)
}
