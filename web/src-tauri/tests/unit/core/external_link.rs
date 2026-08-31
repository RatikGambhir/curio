use super::*;

#[test]
fn only_secure_credential_free_urls_are_external_links() {
    let link = ExternalLink::parse("https://example.com/docs?section=desktop#commands")
        .expect("valid external URL");
    assert_eq!(
        link.as_str(),
        "https://example.com/docs?section=desktop#commands"
    );
    assert!(ExternalLink::parse("http://example.com").is_err());
    assert!(ExternalLink::parse("file:///tmp/curio").is_err());
    assert!(ExternalLink::parse("https://user:secret@example.com").is_err());
}
