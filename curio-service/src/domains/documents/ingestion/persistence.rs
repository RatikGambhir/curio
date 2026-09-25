//! Pure construction of the aggregate write from parser output.
//!
//! These are persistence invariants over parser-derived data, not HTTP request
//! validation: a failure here indicates a parser or embedding defect.
use std::{collections::HashSet, path::Path};

use serde_json::json;

use crate::domains::documents::{
    index::model::{IndexedChunk, validate_chunk_rows},
    model::{
        Document, DocumentChunk, document_id_from_content, file_version_id,
        infer_supported_mime_type, lowercase_extension, sha256_hex,
    },
    store::model::{DocumentGraph, FilePersistence},
};

pub(crate) fn build_document_graph(
    document: Document,
    chunks: Vec<DocumentChunk>,
    file_bytes: Vec<u8>,
) -> Result<DocumentGraph, String> {
    ensure_document_graph_invariants(&document, &chunks)?;
    let file = build_file_persistence_input(&document, file_bytes)?;
    let chunks = chunks
        .into_iter()
        .map(|chunk| build_chunk_row(&file, chunk))
        .collect::<Result<Vec<_>, String>>()?;
    validate_chunk_rows(&file.owner_id, &file.file_id, &file.version_id, &chunks)?;
    Ok(DocumentGraph { file, chunks })
}

pub(crate) fn build_file_persistence_input(
    document: &Document,
    file_bytes: Vec<u8>,
) -> Result<FilePersistence, String> {
    let file_id = normalized_nonempty("file_id", &document.file_id)?;
    let owner_id = normalized_nonempty("owner_id", &document.owner_id)?;
    let display_name = normalized_nonempty("file_name", &document.file_name)?;
    let source_type = normalized_nonempty("source_type", &document.source_type)?;
    let source_uri = document
        .local_path
        .as_deref()
        .map(|path| normalized_nonempty("local_path", path))
        .transpose()?;

    if file_bytes.is_empty() {
        return Err("file bytes cannot be empty".to_string());
    }
    let actual_size = u64::try_from(file_bytes.len())
        .map_err(|_| "file byte length does not fit in u64".to_string())?;
    if actual_size != document.file_size_bytes {
        return Err(format!(
            "file byte size {actual_size} does not match document byte size {}",
            document.file_size_bytes
        ));
    }
    let byte_size = i64::try_from(file_bytes.len())
        .map_err(|_| "file byte length does not fit in bigint".to_string())?;

    let content_sha256 = sha256_hex(&file_bytes);
    if content_sha256 != document.content_hash {
        return Err(format!(
            "file bytes do not match content hash for document `{}`",
            document.document_id
        ));
    }
    let expected_document_id = document_id_from_content(&owner_id, &content_sha256);
    if expected_document_id != document.document_id {
        return Err(format!(
            "file bytes do not match document id `{}`",
            document.document_id
        ));
    }

    let mime_type = infer_supported_mime_type(Path::new(&display_name))
        .ok_or_else(|| format!("unsupported file type for `{display_name}`"))?;
    let extension = lowercase_extension(Path::new(&display_name))
        .ok_or_else(|| format!("unsupported file type for `{display_name}`"))?;
    if source_type.to_ascii_lowercase() != extension {
        return Err(format!(
            "source type `{source_type}` does not match filename `{display_name}`"
        ));
    }

    let metadata_json = serde_json::to_string(&json!({
        "documentId": document.document_id,
        "sourceType": source_type,
        "tokenCount": document.token_count,
        "renderedPdfPath": document.rendered_pdf_path,
    }))
    .map_err(|error| format!("failed to serialize file metadata: {error}"))?;
    let version_id = file_version_id(&file_id, &content_sha256);

    Ok(FilePersistence {
        owner_id,
        file_id,
        display_name,
        source_uri,
        metadata_json,
        version_id,
        mime_type: mime_type.to_string(),
        content_sha256,
        byte_size,
        file_bytes,
    })
}

fn build_chunk_row(file: &FilePersistence, chunk: DocumentChunk) -> Result<IndexedChunk, String> {
    let chunk_index = i64::from(chunk.sequence_number);
    let (page_start, page_end) = page_range(&chunk)?;
    let embedding = chunk
        .embedding
        .ok_or_else(|| format!("chunk `{}` does not contain an embedding", chunk.chunk_id))?;

    Ok(IndexedChunk {
        chunk_id: deterministic_chunk_id(
            &file.owner_id,
            &file.file_id,
            &file.version_id,
            chunk_index,
            &chunk.content_hash,
        ),
        owner_id: file.owner_id.clone(),
        file_id: file.file_id.clone(),
        version_id: file.version_id.clone(),
        chunk_index,
        // PostgreSQL text cannot hold NUL. Replacing the single byte with a
        // single-byte space keeps every stored UTF-8 offset valid.
        text: chunk.text.replace('\0', " "),
        embedding,
        chunk_sha256: chunk.content_hash,
        token_count: i64::from(chunk.token_count),
        page_start,
        page_end,
        char_start: usize_to_i64(chunk.start_offset, "char_start")?,
        char_end: usize_to_i64(chunk.end_offset, "char_end")?,
        section_path: chunk.section_title.unwrap_or_default(),
    })
}

fn deterministic_chunk_id(
    owner_id: &str,
    file_id: &str,
    version_id: &str,
    chunk_index: i64,
    chunk_sha256: &str,
) -> String {
    sha256_hex(
        format!("{owner_id}\0{file_id}\0{version_id}\0{chunk_index}\0{chunk_sha256}").as_bytes(),
    )
}

fn ensure_document_graph_invariants(
    document: &Document,
    chunks: &[DocumentChunk],
) -> Result<(), String> {
    let mut chunk_indices = HashSet::with_capacity(chunks.len());
    let mut embedding_dimension = None;
    for chunk in chunks {
        ensure_chunk_belongs_to_document(document, chunk)?;
        if !chunk_indices.insert(chunk.sequence_number) {
            return Err(format!(
                "document contains duplicate chunk index {}",
                chunk.sequence_number
            ));
        }
        if chunk.start_offset > chunk.end_offset {
            return Err(format!(
                "chunk `{}` has an invalid character range",
                chunk.chunk_id
            ));
        }
        let embedding = chunk
            .embedding
            .as_ref()
            .ok_or_else(|| format!("chunk `{}` does not contain an embedding", chunk.chunk_id))?;
        if embedding.is_empty() {
            return Err(format!("chunk `{}` has an empty embedding", chunk.chunk_id));
        }
        if embedding.iter().any(|value| !value.is_finite()) {
            return Err(format!(
                "chunk `{}` embedding must contain only finite values",
                chunk.chunk_id
            ));
        }
        match embedding_dimension {
            Some(expected) if expected != embedding.len() => {
                return Err(format!(
                    "chunk `{}` embedding dimension {} does not match expected dimension {expected}",
                    chunk.chunk_id,
                    embedding.len()
                ));
            }
            None => embedding_dimension = Some(embedding.len()),
            _ => {}
        }
        page_range(chunk)?;
        usize_to_i64(chunk.start_offset, "char_start")?;
        usize_to_i64(chunk.end_offset, "char_end")?;
    }
    Ok(())
}

fn ensure_chunk_belongs_to_document(
    document: &Document,
    chunk: &DocumentChunk,
) -> Result<(), String> {
    if chunk.document_id != document.document_id {
        return Err(format!(
            "chunk `{}` belongs to document `{}`, not `{}`",
            chunk.chunk_id, chunk.document_id, document.document_id
        ));
    }
    if chunk.owner_id != document.owner_id {
        return Err(format!(
            "chunk `{}` belongs to another owner than its document",
            chunk.chunk_id
        ));
    }
    Ok(())
}

fn page_range(chunk: &DocumentChunk) -> Result<(Option<i64>, Option<i64>), String> {
    let Some(page_numbers) = chunk
        .page_numbers
        .as_ref()
        .filter(|pages| !pages.is_empty())
    else {
        return Ok((None, None));
    };
    let minimum = page_numbers
        .iter()
        .min()
        .copied()
        .ok_or_else(|| format!("chunk `{}` has no minimum page", chunk.chunk_id))?;
    let maximum = page_numbers
        .iter()
        .max()
        .copied()
        .ok_or_else(|| format!("chunk `{}` has no maximum page", chunk.chunk_id))?;
    Ok((Some(i64::from(minimum)), Some(i64::from(maximum))))
}

fn usize_to_i64(value: usize, field: &str) -> Result<i64, String> {
    i64::try_from(value).map_err(|_| format!("{field} value `{value}` does not fit in i64"))
}

fn normalized_nonempty(field: &str, value: &str) -> Result<String, String> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(format!("{field} cannot be empty"));
    }
    if normalized != value {
        return Err(format!("{field} must not contain surrounding whitespace"));
    }
    Ok(normalized.to_string())
}

#[cfg(test)]
#[path = "../../../../tests/unit/domains/documents/ingestion/persistence.rs"]
mod tests;
