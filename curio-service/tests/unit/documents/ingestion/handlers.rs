use super::*;

#[test]
fn accepts_only_trimmed_pdf_and_docx_filenames() {
    for valid in [
        "memo.pdf",
        "memo.PDF",
        "report.docx",
        "Quarterly Report.DocX",
    ] {
        assert!(validate_document_upload_filename(valid).is_ok(), "{valid}");
    }

    for invalid in [
        " memo.pdf",
        "memo.pdf ",
        "memo.doc",
        "memo.txt",
        "memo",
        "me\0mo.pdf",
    ] {
        assert!(
            matches!(
                validate_document_upload_filename(invalid),
                Err(DocumentError::Invalid(_))
            ),
            "{invalid:?}"
        );
    }
}
