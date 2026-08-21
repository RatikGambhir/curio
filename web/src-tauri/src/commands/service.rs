use serde_json::Value;
use tauri::{ipc::Channel, State};

use crate::{
    core::{ServiceMethod, ServicePacket, ServiceRequest},
    services::AppServices,
};

use super::events::ServiceChannel;

#[tauri::command]
pub(crate) async fn service_request(
    services: State<'_, AppServices>,
    request_id: String,
    method: ServiceMethod,
    path: String,
    payload: Option<Value>,
    bearer_token: Option<String>,
    on_packet: Channel<ServicePacket>,
) -> Result<(), String> {
    services
        .service_proxy
        .execute(
            request_id,
            ServiceRequest::new(method, path, payload, bearer_token),
            &ServiceChannel::new(on_packet),
        )
        .await
}

#[tauri::command]
pub(crate) fn cancel_request(
    services: State<'_, AppServices>,
    request_id: String,
) -> Result<bool, String> {
    services.service_proxy.cancel(&request_id)
}
