//! Keep PostgreSQL repository SQL composition behind the shared query layer.
use std::{fs, path::Path};

#[test]
fn postgres_repositories_use_shared_builders_instead_of_inline_sql() {
    fn inspect(directory: &Path, in_repository: bool, inspected: &mut usize) {
        let in_repository = in_repository
            || directory
                .file_name()
                .is_some_and(|name| name == "repository");
        for entry in fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                inspect(&path, in_repository, inspected);
            } else if path.extension().is_some_and(|extension| extension == "rs")
                && (in_repository || path.file_name().is_some_and(|name| name == "repository.rs"))
            {
                *inspected += 1;
                let source = fs::read_to_string(&path).unwrap();
                let compact: String = source.chars().filter(|c| !c.is_whitespace()).collect();
                for forbidden in [
                    "sqlx::query",
                    "sqlx::raw_sql",
                    "QueryBuilder",
                    "\"SELECT",
                    "\"INSERT",
                    "\"UPDATE",
                    "\"DELETE",
                    "\"WITH",
                ] {
                    assert!(
                        !compact.contains(forbidden),
                        "{} bypasses shared query composition with {forbidden}",
                        path.display()
                    );
                }
                if [
                    ".execute(",
                    ".fetch_one(",
                    ".fetch_optional(",
                    ".fetch_all(",
                ]
                .iter()
                .any(|call| compact.contains(call))
                {
                    assert!(
                        compact.contains("query::{"),
                        "{} must use adapters::postgres::query",
                        path.display()
                    );
                }
            }
        }
    }
    let mut inspected = 0;
    inspect(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        false,
        &mut inspected,
    );
    assert!(
        inspected > 0,
        "the repository scan must inspect PostgreSQL adapters"
    );
}
