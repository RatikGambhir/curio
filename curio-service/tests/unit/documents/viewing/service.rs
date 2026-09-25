//! Tests for stored-document viewing services.

use std::io::Cursor;

use docx_rust::{Docx, document::Paragraph};

use super::*;

fn docx_bytes(text: &str) -> Vec<u8> {
    let mut docx = Docx::default();
    docx.document.push(Paragraph::default().push_text(text));
    docx.write(Cursor::new(Vec::new())).unwrap().into_inner()
}

#[derive(Clone, Default)]
struct EmptyStore;

impl StoredDocuments for EmptyStore {
    async fn list(&self, _owner_id: &str) -> Result<Vec<DocumentSummary>, DocumentError> {
        Ok(Vec::new())
    }

    async fn current_blob(
        &self,
        _owner_id: &str,
        _file_id: &str,
    ) -> Result<Option<StoredDocumentBlob>, DocumentError> {
        Ok(None)
    }

    async fn soft_delete(&self, _owner_id: &str, _file_id: &str) -> Result<bool, DocumentError> {
        Ok(false)
    }
}

impl DocumentChunks for EmptyStore {
    async fn current_chunks(
        &self,
        _owner_id: &str,
        _file_id: &str,
    ) -> Result<Option<Vec<ChunkRecord>>, DocumentError> {
        Ok(None)
    }
}

fn service() -> StoredDocumentService<EmptyStore, EmptyStore> {
    StoredDocumentService::new(EmptyStore, EmptyStore, OfficeConverter::new(None))
}

#[tokio::test]
async fn missing_documents_are_not_found_for_every_member_operation() {
    let service = service();
    let not_found = DocumentError::not_found("That document was not found.");

    assert_eq!(
        service.load("user-a", "file-1").await,
        Err(not_found.clone())
    );
    assert_eq!(
        service.chunks("user-a", "file-1").await,
        Err(not_found.clone())
    );
    assert_eq!(service.delete("user-a", "file-1").await, Err(not_found));
    assert!(matches!(
        service.load("user-a", "  ").await,
        Err(DocumentError::Invalid(_))
    ));
}

#[tokio::test]
async fn docx_previews_fall_back_to_rendered_text_without_libreoffice() {
    let pdf = service()
        .render_pdf(StoredDocumentBlob {
            file_id: "file-1".to_string(),
            display_name: "fallback.docx".to_string(),
            mime_type: DOCX_MIME_TYPE.to_string(),
            file_bytes: docx_bytes("A stored DOCX stays previewable without LibreOffice."),
        })
        .await
        .unwrap();

    validate_pdf_bytes(&pdf, "the fallback PDF").unwrap();
    let extracted = pdf_extract::extract_text_from_mem(&pdf).unwrap();
    assert!(extracted.contains("A stored DOCX stays previewable"));
}

#[tokio::test]
async fn unconvertible_office_previews_are_unavailable_without_converter_details() {
    let result = service()
        .render_pdf(StoredDocumentBlob {
            file_id: "file-1".to_string(),
            display_name: "sheet.xlsx".to_string(),
            mime_type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                .to_string(),
            file_bytes: b"not really a workbook".to_vec(),
        })
        .await;

    assert_eq!(
        result,
        Err(DocumentError::Unavailable(
            PREVIEW_FAILED_MESSAGE.to_owned()
        ))
    );
}

#[tokio::test]
async fn unsupported_or_invalid_previews_are_rejected() {
    let unsupported = service()
        .render_pdf(StoredDocumentBlob {
            file_id: "file-1".to_string(),
            display_name: "notes.txt".to_string(),
            mime_type: "text/plain".to_string(),
            file_bytes: b"plain".to_vec(),
        })
        .await;
    assert!(matches!(unsupported, Err(DocumentError::Invalid(_))));

    let corrupt = service()
        .render_pdf(StoredDocumentBlob {
            file_id: "file-1".to_string(),
            display_name: "corrupt.pdf".to_string(),
            mime_type: PDF_MIME_TYPE.to_string(),
            file_bytes: b"not a pdf".to_vec(),
        })
        .await;
    assert!(matches!(corrupt, Err(DocumentError::Invalid(_))));
}

#[test]
fn renders_docx_fallback_pdf_when_office_conversion_is_unavailable() {
    let bytes = docx_bytes("A saved DOCX remains previewable when LibreOffice cannot be started.");

    let pdf = render_office_bytes_as_pdf("docx", "fallback.docx", &bytes, |_, _| {
        Err("LibreOffice/soffice was not found".to_string())
    })
    .unwrap();

    validate_pdf_bytes(&pdf, "the fallback PDF").unwrap();
    let extracted = pdf_extract::extract_text_from_mem(&pdf).unwrap();
    assert!(extracted.contains("A saved DOCX remains previewable"));
}

#[tokio::test]
async fn returns_the_canonical_raw_text_for_a_stored_docx() {
    let response = render_stored_document_as_text(StoredDocumentBlob {
        display_name: "raw.docx".to_string(),
        file_bytes: docx_bytes("Canonical raw text from the stored DOCX."),
        file_id: "file-raw".to_string(),
        mime_type: DOCX_MIME_TYPE.to_string(),
    })
    .await
    .unwrap();

    assert_eq!(response.file_name, "raw.docx");
    assert_eq!(response.source_kind, "docx");
    assert_eq!(response.text, "Canonical raw text from the stored DOCX.");
}

#[test]
fn wraps_and_sanitizes_text_for_the_builtin_pdf_font() {
    let text = format!("{}\n\nSmart “quotes” — and bullets •", "word ".repeat(30));
    let lines = wrap_pdf_text(&text);

    assert!(lines.len() >= 3);
    assert!(
        lines
            .iter()
            .all(|line| line.chars().count() <= PDF_MAX_LINE_CHARACTERS)
    );
    assert_eq!(
        sanitize_pdf_text("Smart “quotes” — bullets •"),
        "Smart \"quotes\" - bullets *"
    );
}

#[test]
fn office_preview_cache_is_bounded_and_evicts_the_oldest_entry() {
    let mut cache = OfficePreviewCache::default();
    for index in 0..=OFFICE_PREVIEW_CACHE_ENTRIES {
        cache.insert(format!("key-{index}"), vec![index as u8]);
    }

    assert_eq!(cache.entries.len(), OFFICE_PREVIEW_CACHE_ENTRIES);
    assert!(!cache.entries.contains_key("key-0"));
    assert!(
        cache
            .entries
            .contains_key(&format!("key-{OFFICE_PREVIEW_CACHE_ENTRIES}"))
    );
}
