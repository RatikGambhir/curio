use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use crate::services::AppServices;

#[tauri::command]
pub(crate) fn open_external_url(
    app: AppHandle,
    services: State<'_, AppServices>,
    url: String,
) -> Result<(), String> {
    let link = services.external_links.validate(&url)?;
    app.opener()
        .open_url(link.as_str(), None::<&str>)
        .map_err(|_| "The desktop app could not open the external link.".to_owned())
}
