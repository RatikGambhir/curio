//! Ingestion values: uploads, per-document outcomes, and background job events.
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct UploadedDocument {
    pub filename: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessedDocument {
    pub filename: String,
    pub document_id: Option<String>,
    pub file_id: Option<String>,
    pub chunk_count: usize,
    pub success: bool,
    pub skipped: bool,
    pub error: Option<String>,
}

impl ProcessedDocument {
    pub fn completed(
        filename: String,
        document_id: String,
        file_id: String,
        chunk_count: usize,
    ) -> Self {
        Self {
            filename,
            document_id: Some(document_id),
            file_id: Some(file_id),
            chunk_count,
            success: true,
            skipped: false,
            error: None,
        }
    }

    pub fn skipped(filename: String, document_id: String, file_id: String) -> Self {
        Self {
            filename,
            document_id: Some(document_id),
            file_id: Some(file_id),
            chunk_count: 0,
            success: true,
            skipped: true,
            error: None,
        }
    }

    pub fn failed(filename: String, error: String) -> Self {
        Self {
            filename,
            document_id: None,
            file_id: None,
            chunk_count: 0,
            success: false,
            skipped: false,
            error: Some(error),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessDocumentsResponse {
    pub total: usize,
    pub succeeded: usize,
    pub skipped: usize,
    pub failed: usize,
    pub documents: Vec<ProcessedDocument>,
}

impl ProcessDocumentsResponse {
    pub fn from_documents(documents: Vec<ProcessedDocument>) -> Self {
        let total = documents.len();
        let succeeded = documents.iter().filter(|document| document.success).count();
        let skipped = documents.iter().filter(|document| document.skipped).count();
        Self {
            total,
            succeeded,
            skipped,
            failed: total - succeeded,
            documents,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartedDocumentJob {
    pub job_id: String,
    pub filename: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentJobEvent {
    pub job_id: String,
    pub filename: String,
    pub status: DocumentJobStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DocumentJobStatus {
    Processing,
    Completed,
    Skipped,
    Failed,
}

impl DocumentJobEvent {
    pub fn processing(job_id: String, filename: String) -> Self {
        Self {
            job_id,
            filename,
            status: DocumentJobStatus::Processing,
            document_id: None,
            file_id: None,
            chunk_count: None,
            error: None,
        }
    }

    /// Maps a finished document outcome onto the job's terminal event.
    pub fn finished(job_id: String, document: ProcessedDocument) -> Self {
        let (status, chunk_count) = if document.skipped {
            (DocumentJobStatus::Skipped, None)
        } else if document.success {
            (DocumentJobStatus::Completed, Some(document.chunk_count))
        } else {
            (DocumentJobStatus::Failed, None)
        };
        Self {
            job_id,
            filename: document.filename,
            status,
            document_id: document.document_id,
            file_id: document.file_id,
            chunk_count,
            error: document.error,
        }
    }

    pub fn event_name(&self) -> &'static str {
        match self.status {
            DocumentJobStatus::Processing => "processing",
            DocumentJobStatus::Completed => "completed",
            DocumentJobStatus::Skipped => "skipped",
            DocumentJobStatus::Failed => "failed",
        }
    }

    pub fn is_terminal(&self) -> bool {
        !matches!(self.status, DocumentJobStatus::Processing)
    }
}
