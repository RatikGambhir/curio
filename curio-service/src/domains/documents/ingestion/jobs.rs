//! Background single-document ingestion jobs with owner-scoped status streams.
//!
//! Job state lives only in this process. Restarting the service loses running
//! jobs, and finished jobs are forgotten after the configured retention.
use std::{collections::HashMap, sync::Arc, time::Duration};

use tokio::sync::{RwLock, watch};
use uuid::Uuid;

use super::{
    model::{DocumentJobEvent, ProcessedDocument, StartedDocumentJob, UploadedDocument},
    service::{IngestionService, IngestionStore},
};
use crate::domains::documents::model::{DocumentError, Embedder};

struct JobEntry {
    owner_id: String,
    sender: watch::Sender<DocumentJobEvent>,
}

#[derive(Clone)]
pub struct DocumentJobService<S, E> {
    ingestion: IngestionService<S, E>,
    jobs: Arc<RwLock<HashMap<String, JobEntry>>>,
    completed_retention: Duration,
}

impl<S: IngestionStore, E: Embedder> DocumentJobService<S, E> {
    pub fn new(ingestion: IngestionService<S, E>, completed_retention: Duration) -> Self {
        Self {
            ingestion,
            jobs: Arc::new(RwLock::new(HashMap::new())),
            completed_retention,
        }
    }

    pub async fn start(&self, owner_id: &str, file: UploadedDocument) -> StartedDocumentJob {
        let filename = file.filename.clone();
        let job_id = Uuid::new_v4().to_string();
        let (sender, _) = watch::channel(DocumentJobEvent::processing(
            job_id.clone(),
            filename.clone(),
        ));
        self.jobs.write().await.insert(
            job_id.clone(),
            JobEntry {
                owner_id: owner_id.to_owned(),
                sender,
            },
        );

        let service = self.clone();
        let worker_owner_id = owner_id.to_owned();
        let worker_job_id = job_id.clone();
        let worker_filename = filename.clone();
        tokio::spawn(async move {
            let document = service
                .ingestion
                .process(&worker_owner_id, vec![file])
                .await
                .documents
                .into_iter()
                .next()
                .unwrap_or_else(|| {
                    ProcessedDocument::failed(
                        worker_filename,
                        "Document processing returned no result.".to_owned(),
                    )
                });
            let event = DocumentJobEvent::finished(worker_job_id.clone(), document);
            if let Some(entry) = service.jobs.read().await.get(&worker_job_id) {
                entry.sender.send_replace(event);
            }
            tokio::time::sleep(service.completed_retention).await;
            service.jobs.write().await.remove(&worker_job_id);
        });

        StartedDocumentJob { job_id, filename }
    }

    /// Subscribes the job's owner to its status. Unknown and foreign jobs are
    /// indistinguishable so job ids cannot be probed across owners.
    pub async fn subscribe(
        &self,
        owner_id: &str,
        job_id: &str,
    ) -> Result<watch::Receiver<DocumentJobEvent>, DocumentError> {
        self.jobs
            .read()
            .await
            .get(job_id)
            .filter(|entry| entry.owner_id == owner_id)
            .map(|entry| entry.sender.subscribe())
            .ok_or_else(|| DocumentError::not_found("That document job was not found."))
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/domains/documents/ingestion/jobs.rs"]
mod tests;
