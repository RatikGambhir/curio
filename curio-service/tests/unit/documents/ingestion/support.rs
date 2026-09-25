//! In-memory ports for ingestion use-case tests.
use std::{
    io::Cursor,
    sync::{Arc, Mutex},
};

use docx_rust::{Docx, document::Paragraph};

use crate::documents::{
    domain::{DocumentError, Embedder},
    ingestion::service::IngestionStore,
    store::domain::{DocumentGraph, PersistedFileIdentity},
};

#[derive(Clone, Default)]
pub struct MemoryStore {
    pub graphs: Arc<Mutex<Vec<DocumentGraph>>>,
    pub fail_persist: Option<DocumentError>,
}

impl IngestionStore for MemoryStore {
    async fn find_current_by_hash(
        &self,
        owner_id: &str,
        content_sha256: &str,
    ) -> Result<Option<PersistedFileIdentity>, DocumentError> {
        Ok(self
            .graphs
            .lock()
            .unwrap()
            .iter()
            .find(|graph| {
                graph.file.owner_id == owner_id && graph.file.content_sha256 == content_sha256
            })
            .map(identity))
    }

    async fn persist(&self, graph: DocumentGraph) -> Result<PersistedFileIdentity, DocumentError> {
        if let Some(error) = &self.fail_persist {
            return Err(error.clone());
        }
        let identity = identity(&graph);
        self.graphs.lock().unwrap().push(graph);
        Ok(identity)
    }
}

fn identity(graph: &DocumentGraph) -> PersistedFileIdentity {
    PersistedFileIdentity {
        file_id: graph.file.file_id.clone(),
        owner_id: graph.file.owner_id.clone(),
        display_name: graph.file.display_name.clone(),
        version_id: graph.file.version_id.clone(),
    }
}

/// Returns a two-dimensional vector per input, or a fixed sanitized error.
#[derive(Clone, Default)]
pub struct FixedEmbedder {
    pub error: Option<&'static str>,
    pub calls: Arc<Mutex<Vec<Vec<String>>>>,
}

impl Embedder for FixedEmbedder {
    async fn embed(&self, inputs: Vec<String>) -> Result<Vec<Vec<f32>>, String> {
        self.calls.lock().unwrap().push(inputs.clone());
        match self.error {
            Some(error) => Err(error.to_owned()),
            None => Ok(inputs
                .iter()
                .map(|input| vec![input.len() as f32, 1.0])
                .collect()),
        }
    }
}

pub fn docx_bytes(text: &str) -> Vec<u8> {
    let mut docx = Docx::default();
    docx.document.push(Paragraph::default().push_text(text));
    docx.write(Cursor::new(Vec::new())).unwrap().into_inner()
}
