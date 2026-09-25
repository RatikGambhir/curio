//! Parser and chunker behavior through the public `documents::formats` API.
//! These tests need no database.
use curio_service::domains::documents::{
    formats::{
        ParsedSourceFile, SourceFile,
        docx::{parse_docx_chunks_from_bytes, parse_docx_from_bytes},
        pdf::{parse_pdf_by_bytes, parse_pdf_from_bytes},
        text_chunking::{MAX_TOKEN_CHUNK, token_bounded_ranges},
    },
    model::{
        DocumentChunk, document_id_from_content, infer_supported_mime_type,
        office_extension_for_mime_type, sha256_hex,
    },
};
use std::path::Path;

use crate::support::{docx_bytes, long_text, pdf_bytes};

fn parse(file_name: &str, bytes: Vec<u8>, owner: &str) -> Result<ParsedSourceFile, String> {
    SourceFile::from_bytes(file_name, bytes)?.parse(owner)
}

fn chunks_of(parsed: ParsedSourceFile) -> (String, Vec<DocumentChunk>) {
    match parsed {
        ParsedSourceFile::Pdf(assembly) => (assembly.document.document_id, assembly.chunks),
        ParsedSourceFile::Docx(assembly) => (assembly.document.document_id, assembly.chunks),
    }
}

/// Every chunk is a contiguous, in-budget slice and together they rebuild `text`.
fn assert_chunk_invariants(text: &str, chunks: &[DocumentChunk]) {
    let mut expected_start = 0;
    for (index, chunk) in chunks.iter().enumerate() {
        assert_eq!(chunk.sequence_number as usize, index + 1);
        assert_eq!(
            chunk.start_offset, expected_start,
            "chunks must be contiguous"
        );
        assert_eq!(&text[chunk.start_offset..chunk.end_offset], chunk.text);
        assert!(chunk.token_count as usize <= MAX_TOKEN_CHUNK);
        assert!(chunk.token_count > 0);
        assert_eq!(chunk.content_hash, sha256_hex(chunk.text.as_bytes()));
        assert!(chunk.embedding.is_none(), "parsers never embed");
        expected_start = chunk.end_offset;
    }
    assert_eq!(expected_start, text.len());
}

#[test]
fn source_files_accept_only_nonempty_pdf_and_docx_names() {
    assert_eq!(
        SourceFile::from_bytes("memo.pdf", Vec::new()).unwrap_err(),
        "file is empty"
    );
    for unsupported in ["notes.txt", "sheet.xlsx", "legacy.doc", "no-extension"] {
        assert_eq!(
            SourceFile::from_bytes(unsupported, b"bytes".to_vec()).unwrap_err(),
            "invalid file format",
            "{unsupported}"
        );
    }
    assert!(matches!(
        SourceFile::from_bytes("REPORT.PDF", b"bytes".to_vec()).unwrap(),
        SourceFile::Pdf { .. }
    ));
    assert!(matches!(
        SourceFile::from_bytes("Report.DocX", b"bytes".to_vec()).unwrap(),
        SourceFile::Docx { .. }
    ));
}

#[test]
fn parsing_requires_an_owner_and_trims_it() {
    for blank in ["", "   "] {
        assert_eq!(
            parse("memo.docx", docx_bytes(&["Body."]), blank).unwrap_err(),
            "owner_id cannot be empty"
        );
    }

    let ParsedSourceFile::Docx(assembly) =
        parse("memo.docx", docx_bytes(&["Body."]), "  user-a  ").unwrap()
    else {
        panic!("expected a DOCX assembly");
    };
    assert_eq!(assembly.document.owner_id, "user-a");
    assert!(
        assembly
            .chunks
            .iter()
            .all(|chunk| chunk.owner_id == "user-a")
    );
}

#[test]
fn corrupt_files_are_errors_not_panics() {
    assert!(parse("broken.pdf", b"%PDF-1.5 truncated".to_vec(), "user-a").is_err());
    assert!(parse("broken.pdf", b"not a pdf at all".to_vec(), "user-a").is_err());
    assert!(parse("broken.docx", b"PK\x03\x04 not a zip".to_vec(), "user-a").is_err());
    assert!(parse("broken.docx", pdf_bytes(&["A PDF named .docx"]), "user-a").is_err());
}

#[test]
fn a_docx_without_readable_text_is_rejected() {
    assert_eq!(
        parse_docx_from_bytes(docx_bytes(&[])).unwrap_err(),
        "DOCX did not contain readable text"
    );
    assert_eq!(
        parse_docx_from_bytes(docx_bytes(&["   ", "\t"])).unwrap_err(),
        "DOCX did not contain readable text"
    );
}

#[test]
fn pdf_pages_are_joined_with_blank_pages_skipped_but_numbering_kept() {
    let bytes = pdf_bytes(&["First page.", "", "Third page."]);

    let assembly = parse_pdf_by_bytes(bytes.clone(), None, "user-a").unwrap();
    let canonical = parse_pdf_from_bytes(&bytes).unwrap();

    assert_eq!(canonical, "First page.\n\nThird page.");
    assert_eq!(assembly.chunks.len(), 1);
    assert_eq!(assembly.chunks[0].page_numbers, Some(vec![1, 3]));
    assert_chunk_invariants(&canonical, &assembly.chunks);
    assert_eq!(assembly.document.file_size_bytes, bytes.len() as u64);
    assert_eq!(assembly.document.content_hash, sha256_hex(&bytes));
}

#[test]
fn a_pdf_without_text_parses_to_zero_chunks() {
    let assembly = parse_pdf_by_bytes(pdf_bytes(&["", ""]), None, "user-a").unwrap();

    assert!(assembly.chunks.is_empty());
    assert_eq!(assembly.document.token_count, 0);
}

#[test]
fn long_pdf_pages_split_into_chunks_that_report_their_pages() {
    let page = long_text("Quarterly revenue grew in every region. ", 2);
    let bytes = pdf_bytes(&[&page, "Closing page."]);

    let assembly = parse_pdf_by_bytes(bytes.clone(), None, "user-a").unwrap();
    let canonical = parse_pdf_from_bytes(&bytes).unwrap();

    assert!(assembly.chunks.len() > 2, "{}", assembly.chunks.len());
    assert_chunk_invariants(&canonical, &assembly.chunks);
    assert_eq!(assembly.chunks[0].page_numbers, Some(vec![1]));
    let last_pages = assembly
        .chunks
        .last()
        .unwrap()
        .page_numbers
        .clone()
        .unwrap();
    assert!(last_pages.contains(&2), "{last_pages:?}");
    assert_eq!(
        assembly.document.token_count,
        assembly
            .chunks
            .iter()
            .map(|chunk| u64::from(chunk.token_count))
            .sum::<u64>()
    );
}

#[test]
fn long_docx_text_splits_into_in_budget_chunks() {
    let body = long_text("The committee reviewed the annual plan. ", 3);
    let bytes = docx_bytes(&[&body, "Final paragraph."]);

    let assembly = parse_docx_chunks_from_bytes(bytes.clone(), None, "user-a").unwrap();
    let canonical = parse_docx_from_bytes(bytes).unwrap();

    assert!(assembly.chunks.len() > 3);
    assert_chunk_invariants(&canonical, &assembly.chunks);
    assert!(
        assembly
            .chunks
            .iter()
            .all(|chunk| chunk.page_numbers.is_none())
    );
    assert!(canonical.ends_with("Final paragraph."));
}

#[test]
fn multibyte_text_is_never_split_inside_a_character() {
    let text = "Résumé 数据 🏗️ ünïcödé — ".repeat(3_000);

    let ranges = token_bounded_ranges(&text);

    assert!(ranges.len() > 1);
    let mut expected_start = 0;
    for range in &ranges {
        assert_eq!(range.start_offset, expected_start);
        assert!(text.is_char_boundary(range.start_offset));
        assert!(text.is_char_boundary(range.end_offset));
        assert!(range.token_count <= MAX_TOKEN_CHUNK);
        expected_start = range.end_offset;
    }
    assert_eq!(expected_start, text.len());
}

#[test]
fn chunking_handles_empty_tiny_and_unbroken_text() {
    assert!(token_bounded_ranges("").is_empty());

    let single = token_bounded_ranges("x");
    assert_eq!(single.len(), 1);
    assert_eq!((single[0].start_offset, single[0].end_offset), (0, 1));

    // No whitespace at all still splits within the budget.
    let unbroken = "a1b2c3d4".repeat(20_000);
    let ranges = token_bounded_ranges(&unbroken);
    assert!(ranges.len() > 1);
    assert!(
        ranges
            .iter()
            .all(|range| range.token_count <= MAX_TOKEN_CHUNK)
    );
    assert_eq!(ranges.last().unwrap().end_offset, unbroken.len());
}

#[test]
fn chunk_boundaries_are_maximal_within_the_token_budget() {
    let text = long_text("Growth continued across all segments. ", 3);
    let ranges = token_bounded_ranges(&text);

    // Every chunk but the last would exceed the budget with one more character.
    let tokenizer = tiktoken_rs::o200k_base().unwrap();
    assert!(ranges.len() > 1);
    for range in &ranges[..ranges.len() - 1] {
        let next_char = text[range.end_offset..].chars().next().unwrap();
        let extended = &text[range.start_offset..range.end_offset + next_char.len_utf8()];
        assert!(tokenizer.count_with_special_tokens(extended) > MAX_TOKEN_CHUNK);
    }
}

#[test]
fn identities_are_deterministic_per_owner_and_content() {
    let bytes = docx_bytes(&["Stable identity body."]);

    let (first_document, first_chunks) =
        chunks_of(parse("a.docx", bytes.clone(), "user-a").unwrap());
    let (renamed_document, renamed_chunks) =
        chunks_of(parse("renamed.docx", bytes.clone(), "user-a").unwrap());
    let (other_owner_document, other_owner_chunks) =
        chunks_of(parse("a.docx", bytes.clone(), "user-b").unwrap());

    assert_eq!(
        first_document, renamed_document,
        "the filename is not identity"
    );
    assert_eq!(first_chunks[0].chunk_id, renamed_chunks[0].chunk_id);
    assert_eq!(
        first_document,
        document_id_from_content("user-a", &sha256_hex(&bytes))
    );
    assert_ne!(first_document, other_owner_document);
    assert_ne!(first_chunks[0].chunk_id, other_owner_chunks[0].chunk_id);

    // A logical file id is random per parse; persistence may reuse an existing one.
    let ParsedSourceFile::Docx(one) = parse("a.docx", bytes.clone(), "user-a").unwrap() else {
        panic!("expected DOCX");
    };
    let ParsedSourceFile::Docx(two) = parse("a.docx", bytes, "user-a").unwrap() else {
        panic!("expected DOCX");
    };
    assert_ne!(one.document.file_id, two.document.file_id);
}

#[test]
fn file_policy_maps_extensions_and_office_mime_types_case_insensitively() {
    assert_eq!(
        infer_supported_mime_type(Path::new("REPORT.PDF")),
        Some("application/pdf")
    );
    assert_eq!(
        infer_supported_mime_type(Path::new("memo.DocX")),
        Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document")
    );
    assert_eq!(infer_supported_mime_type(Path::new("archive.zip")), None);
    assert_eq!(infer_supported_mime_type(Path::new("no-extension")), None);

    for extension in ["doc", "docx", "xls", "xlsx", "pptx"] {
        let mime = infer_supported_mime_type(Path::new(&format!("file.{extension}"))).unwrap();
        assert_eq!(office_extension_for_mime_type(mime), Some(extension));
    }
    assert_eq!(office_extension_for_mime_type("application/pdf"), None);
}
