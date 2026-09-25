use super::*;

const OWNER: &str = "user-a";

fn document(file_id: &str, owner: &str, filename: &str, bytes: &[u8]) -> Document {
    let content_hash = sha256_hex(bytes);
    Document {
        file_id: file_id.to_string(),
        document_id: document_id_from_content(owner, &content_hash),
        owner_id: owner.to_string(),
        file_name: filename.to_string(),
        source_type: Path::new(filename)
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase(),
        local_path: Some(format!("/documents/{filename}")),
        file_size_bytes: bytes.len() as u64,
        token_count: 42,
        content_hash,
        rendered_pdf_path: Some(format!("/rendered/{filename}.pdf")),
    }
}

fn chunk(document: &Document, sequence_number: u32, embedding: Option<Vec<f32>>) -> DocumentChunk {
    DocumentChunk {
        chunk_id: format!("transient-chunk-{sequence_number}"),
        document_id: document.document_id.clone(),
        owner_id: document.owner_id.clone(),
        text: "graph".to_string(),
        embedding,
        sequence_number,
        page_numbers: Some(vec![3, 2, 3]),
        start_offset: 1,
        end_offset: 6,
        token_count: 1,
        content_hash: sha256_hex(b"graph"),
        section_title: Some("Overview".to_string()),
    }
}

#[test]
fn file_persistence_maps_document_metadata_and_derives_the_version() {
    let bytes = b"file version node";
    let document = document("logical-file", OWNER, "report.pdf", bytes);

    let file = build_file_persistence_input(&document, bytes.to_vec()).unwrap();

    assert_eq!(file.owner_id, OWNER);
    assert_eq!(file.file_id, "logical-file");
    assert_eq!(file.display_name, "report.pdf");
    assert_eq!(file.source_uri.as_deref(), Some("/documents/report.pdf"));
    assert_eq!(file.mime_type, "application/pdf");
    assert_eq!(file.content_sha256, document.content_hash);
    assert_eq!(file.byte_size, bytes.len() as i64);
    assert_eq!(
        file.version_id,
        file_version_id(&document.file_id, &document.content_hash)
    );
    let metadata: serde_json::Value = serde_json::from_str(&file.metadata_json).unwrap();
    assert_eq!(metadata["documentId"], document.document_id);
    assert_eq!(metadata["tokenCount"], 42);
}

#[test]
fn file_persistence_rejects_bytes_that_do_not_match_the_document() {
    let document = document("logical-file", OWNER, "report.pdf", b"original bytes");

    assert!(build_file_persistence_input(&document, b"changed bytes".to_vec()).is_err());
    assert!(build_file_persistence_input(&document, Vec::new()).is_err());

    let mut mismatched_type = document.clone();
    mismatched_type.source_type = "docx".to_string();
    assert!(build_file_persistence_input(&mismatched_type, b"original bytes".to_vec()).is_err());

    let mut padded_owner = document;
    padded_owner.owner_id = format!(" {OWNER}");
    assert!(build_file_persistence_input(&padded_owner, b"original bytes".to_vec()).is_err());
}

#[test]
fn graph_maps_chunks_to_deterministic_version_scoped_rows() {
    let bytes = b"graph document";
    let document = document("logical-file", OWNER, "report.pdf", bytes);
    let service_chunk = chunk(&document, 7, Some(vec![0.25, 0.5]));

    let first = build_document_graph(
        document.clone(),
        vec![service_chunk.clone()],
        bytes.to_vec(),
    )
    .unwrap();
    let retry =
        build_document_graph(document.clone(), vec![service_chunk], bytes.to_vec()).unwrap();
    let row = &first.chunks[0];

    assert_eq!(row.chunk_id, retry.chunks[0].chunk_id);
    assert_eq!(
        row.chunk_id,
        deterministic_chunk_id(
            OWNER,
            &first.file.file_id,
            &first.file.version_id,
            7,
            &sha256_hex(b"graph"),
        )
    );
    assert_ne!(
        row.chunk_id,
        deterministic_chunk_id(
            OWNER,
            "another-file",
            &first.file.version_id,
            7,
            &row.chunk_sha256
        )
    );
    assert_eq!(row.owner_id, OWNER);
    assert_eq!(row.file_id, first.file.file_id);
    assert_eq!(row.version_id, first.file.version_id);
    assert_eq!(row.chunk_index, 7);
    assert_eq!(row.text, "graph");
    assert_eq!(row.embedding, vec![0.25, 0.5]);
    assert_eq!(row.token_count, 1);
    assert_eq!(row.page_start, Some(2));
    assert_eq!(row.page_end, Some(3));
    assert_eq!(row.char_start, 1);
    assert_eq!(row.char_end, 6);
    assert_eq!(row.section_path, "Overview");
}

#[test]
fn graph_replaces_nul_bytes_without_moving_offsets() {
    let bytes = b"nul document";
    let document = document("logical-file", OWNER, "report.pdf", bytes);
    let mut service_chunk = chunk(&document, 1, Some(vec![1.0]));
    service_chunk.text = "a\0b".to_string();

    let graph = build_document_graph(document, vec![service_chunk], bytes.to_vec()).unwrap();

    assert_eq!(graph.chunks[0].text, "a b");
    assert_eq!(graph.chunks[0].text.len(), 3);
}

#[test]
fn graph_rejects_invalid_chunk_metadata_before_persisting() {
    let bytes = b"invalid graph document";
    let document = document("logical-file", OWNER, "report.docx", bytes);
    let build = |chunks| build_document_graph(document.clone(), chunks, bytes.to_vec());

    assert!(build(vec![chunk(&document, 1, None)]).is_err());
    assert!(
        build(vec![
            chunk(&document, 1, Some(vec![1.0])),
            chunk(&document, 1, Some(vec![1.0])),
        ])
        .is_err()
    );
    assert!(
        build(vec![
            chunk(&document, 1, Some(vec![1.0])),
            chunk(&document, 2, Some(vec![1.0, 2.0])),
        ])
        .is_err()
    );
    assert!(build(vec![chunk(&document, 1, Some(vec![f32::NAN]))]).is_err());

    let mut without_pages = chunk(&document, 1, Some(vec![1.0]));
    without_pages.page_numbers = None;
    let graph = build(vec![without_pages]).unwrap();
    assert_eq!(graph.chunks[0].page_start, None);
    assert_eq!(graph.chunks[0].page_end, None);
}

#[test]
fn rejects_a_chunk_for_another_document_before_persisting() {
    let document = document("file-1", OWNER, "test.pdf", b"one");
    let mut foreign = chunk(&document, 1, Some(vec![1.0]));
    foreign.document_id = "doc-2".to_string();

    assert!(ensure_chunk_belongs_to_document(&document, &foreign).is_err());

    let mut other_owner = chunk(&document, 1, Some(vec![1.0]));
    other_owner.owner_id = "user-b".to_string();
    assert!(ensure_chunk_belongs_to_document(&document, &other_owner).is_err());
}
