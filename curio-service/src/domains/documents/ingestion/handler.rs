use std::{convert::Infallible, path::Path as FilePath, sync::Arc, time::Duration};

use axum::{
    Extension, Json,
    extract::{Multipart, Path, State, multipart::MultipartError},
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
};
use futures_util::stream::{self, Stream};

use super::model::{ProcessDocumentsResponse, StartedDocumentJob, UploadedDocument};
use crate::{
    domains::documents::{
        Ingestion, Jobs,
        model::{DocumentError, MAX_FILE_BYTES, MAX_TOTAL_REQUEST_FILE_BYTES, lowercase_extension},
    },
    shared::auth::CurrentUser,
};

const FILE_FIELD: &str = "files";
const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(15);

pub async fn process_documents(
    State(ingestion): State<Arc<Ingestion>>,
    Extension(current_user): Extension<CurrentUser>,
    multipart: Multipart,
) -> Result<Json<ProcessDocumentsResponse>, DocumentError> {
    let files = collect_document_upload(multipart).await?;
    Ok(Json(ingestion.process(current_user.id(), files).await))
}

pub async fn start_job(
    State(jobs): State<Arc<Jobs>>,
    Extension(current_user): Extension<CurrentUser>,
    multipart: Multipart,
) -> Result<(StatusCode, Json<StartedDocumentJob>), DocumentError> {
    let mut files = collect_document_upload(multipart).await?;
    if files.len() != 1 {
        return Err(DocumentError::BadRequest(
            "A document job accepts exactly one file.".to_owned(),
        ));
    }
    let started = jobs.start(current_user.id(), files.remove(0)).await;
    Ok((StatusCode::ACCEPTED, Json(started)))
}

pub async fn job_events(
    State(jobs): State<Arc<Jobs>>,
    Extension(current_user): Extension<CurrentUser>,
    Path(job_id): Path<String>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, DocumentError> {
    let receiver = jobs.subscribe(current_user.id(), job_id.trim()).await?;

    // Emit the current state immediately, then each change until terminal.
    let events = stream::unfold(
        (receiver, true, false),
        |(mut receiver, initial, finished)| async move {
            if finished {
                return None;
            }
            if !initial && receiver.changed().await.is_err() {
                return None;
            }

            let status = receiver.borrow_and_update().clone();
            let finished = status.is_terminal();
            let event = Event::default()
                .event(status.event_name())
                .json_data(status)
                .expect("document job events are JSON serializable");
            Some((Ok(event), (receiver, false, finished)))
        },
    );

    Ok(Sse::new(events).keep_alive(
        KeepAlive::new()
            .interval(KEEP_ALIVE_INTERVAL)
            .text("keep-alive"),
    ))
}

async fn collect_document_upload(
    mut multipart: Multipart,
) -> Result<Vec<UploadedDocument>, DocumentError> {
    let mut files = Vec::new();
    let mut total_bytes = 0usize;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|error| multipart_error(error, "The upload could not be read."))?
    {
        let name = field.name().unwrap_or_default().to_string();
        if matches!(name.as_str(), "userId" | "user_id" | "ownerId" | "owner_id") {
            return Err(DocumentError::BadRequest(
                "The document owner comes from the Authorization header.".to_owned(),
            ));
        }
        if name != FILE_FIELD {
            continue;
        }

        let filename = field
            .file_name()
            .map(str::to_string)
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| DocumentError::invalid("Every file must include a filename."))?;
        validate_document_upload_filename(&filename)?;
        let bytes = field.bytes().await.map_err(|error| {
            multipart_error(
                error,
                &format!("The upload `{filename}` could not be read."),
            )
        })?;
        if bytes.is_empty() {
            return Err(DocumentError::invalid(format!(
                "The upload `{filename}` is empty."
            )));
        }
        if bytes.len() > MAX_FILE_BYTES {
            return Err(DocumentError::PayloadTooLarge(format!(
                "The upload `{filename}` exceeds the 50 MB file limit."
            )));
        }
        total_bytes = total_bytes.saturating_add(bytes.len());
        if total_bytes > MAX_TOTAL_REQUEST_FILE_BYTES {
            return Err(DocumentError::PayloadTooLarge(
                "The uploaded files exceed the 50 MB request limit.".to_owned(),
            ));
        }
        files.push(UploadedDocument {
            filename,
            bytes: bytes.to_vec(),
        });
    }

    if files.is_empty() {
        return Err(DocumentError::invalid(
            "At least one PDF or DOCX upload is required in the `files` field.",
        ));
    }
    Ok(files)
}

fn multipart_error(error: MultipartError, message: &str) -> DocumentError {
    if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
        DocumentError::PayloadTooLarge("The upload exceeds the 50 MB request limit.".to_owned())
    } else {
        DocumentError::BadRequest(message.to_owned())
    }
}

fn validate_document_upload_filename(filename: &str) -> Result<(), DocumentError> {
    if filename.trim() != filename {
        return Err(DocumentError::invalid(
            "Upload filenames must not contain surrounding whitespace.",
        ));
    }
    if filename.contains('\0') {
        return Err(DocumentError::invalid(
            "Upload filenames must not contain NUL characters.",
        ));
    }
    if !matches!(
        lowercase_extension(FilePath::new(filename)).as_deref(),
        Some("pdf" | "docx")
    ) {
        return Err(DocumentError::invalid(format!(
            "The upload `{filename}` must be a PDF or DOCX file."
        )));
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../../tests/unit/domains/documents/ingestion/handler.rs"]
mod tests;
