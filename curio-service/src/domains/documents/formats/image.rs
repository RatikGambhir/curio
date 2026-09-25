use std::{fs, path::Path};

use base64::{Engine as _, engine::general_purpose};

use super::image_prompt::IMAGE_DESCRIPTION_PROMPT;
use crate::domains::documents::model::{ImageDescriber, lowercase_extension};

pub async fn parse_image_file(
    image_path: &Path,
    describer: &impl ImageDescriber,
) -> Result<String, String> {
    let image = fs::read(image_path)
        .map_err(|err| format!("failed to read image file {}: {err}", image_path.display()))?;
    let mime_type = infer_image_mime_type(image_path)?;

    describe_image(&image, mime_type, describer).await
}

pub async fn describe_image(
    image: &[u8],
    mime_type: &str,
    describer: &impl ImageDescriber,
) -> Result<String, String> {
    if image.is_empty() {
        return Err("cannot describe an empty image".to_string());
    }

    let normalized_mime_type = normalize_image_mime_type(mime_type)?;
    let image_base64 = general_purpose::STANDARD.encode(image);
    let description = describer
        .describe_image(
            IMAGE_DESCRIPTION_PROMPT,
            normalized_mime_type,
            &image_base64,
        )
        .await?;
    let description = description.trim().to_string();

    if description.is_empty() {
        return Err("image analysis returned an empty description".to_string());
    }

    Ok(description)
}

fn infer_image_mime_type(image_path: &Path) -> Result<&'static str, String> {
    match lowercase_extension(image_path).as_deref() {
        Some("png") => Ok("image/png"),
        Some("jpg") | Some("jpeg") => Ok("image/jpeg"),
        Some("webp") => Ok("image/webp"),
        Some("gif") => Ok("image/gif"),
        Some(extension) => Err(format!(
            "unsupported image extension .{extension}; expected png, jpg, jpeg, webp, or gif"
        )),
        None => Err(format!(
            "could not infer image type for {}; pass bytes with an explicit MIME type instead",
            image_path.display()
        )),
    }
}

fn normalize_image_mime_type(mime_type: &str) -> Result<&'static str, String> {
    match mime_type.trim().to_ascii_lowercase().as_str() {
        "image/png" => Ok("image/png"),
        "image/jpeg" | "image/jpg" => Ok("image/jpeg"),
        "image/webp" => Ok("image/webp"),
        "image/gif" => Ok("image/gif"),
        value => Err(format!(
            "unsupported image MIME type {value}; expected image/png, image/jpeg, image/webp, or image/gif"
        )),
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/domains/documents/formats/image.rs"]
mod tests;
