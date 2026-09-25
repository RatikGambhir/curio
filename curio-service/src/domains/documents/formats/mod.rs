//! Source-format parsers that turn file bytes into graph-ready documents and
//! token-bounded chunks. Parsers are synchronous and CPU-bound; callers run
//! them on a blocking thread.
pub mod docx;
pub mod image;
pub mod image_prompt;
pub mod pdf;
pub mod powerpoint;
pub mod spreadsheet;
pub mod text_chunking;

use std::{
    fs::{self, File, Metadata},
    io::Read,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

use self::{
    docx::{DocxAssembly, parse_docx_chunks_from_bytes},
    pdf::{PdfDocumentAssembly, parse_pdf_by_bytes},
};
use super::model::lowercase_extension;

/// A supported source file awaiting parsing.
#[derive(Debug)]
pub enum SourceFile {
    Pdf {
        bytes: Vec<u8>,
        path: Option<PathBuf>,
        file_name: String,
    },
    Docx {
        bytes: Vec<u8>,
        path: Option<PathBuf>,
        file_name: String,
    },
}

#[derive(Debug)]
pub enum ParsedSourceFile {
    Pdf(PdfDocumentAssembly),
    Docx(DocxAssembly),
}

/// Reads filesystem metadata from an already-open file without consuming it.
pub fn generate_file_metadata(file: &File) -> Result<Metadata, String> {
    file.metadata()
        .map_err(|error| format!("failed to read file metadata: {error}"))
}

impl SourceFile {
    pub fn from_local_path(path: impl Into<PathBuf>) -> Result<Self, String> {
        let path = path.into();
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("failed to derive filename from {}", path.display()))?
            .to_string();
        let bytes = fs::read(&path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?;

        Self::from_parts(file_name, bytes, Some(path))
    }

    /// Builds a parser input from uploaded bytes without inventing a
    /// server-local path for the document.
    pub fn from_bytes(file_name: impl Into<String>, bytes: Vec<u8>) -> Result<Self, String> {
        Self::from_parts(file_name.into(), bytes, None)
    }

    /// Parses the file into its document/chunk assembly. Owner identity is
    /// explicit so parsed records are never produced unscoped.
    pub fn parse(self, owner_id: &str) -> Result<ParsedSourceFile, String> {
        let owner_id = owner_id.trim();
        if owner_id.is_empty() {
            return Err("owner_id cannot be empty".to_string());
        }

        match self {
            Self::Pdf {
                bytes,
                path,
                file_name,
            } => {
                let mut assembly = parse_pdf_by_bytes(bytes, path.as_deref(), owner_id)?;
                assembly.document.file_name = file_name;
                Ok(ParsedSourceFile::Pdf(assembly))
            }
            Self::Docx {
                bytes,
                path,
                file_name,
            } => {
                let mut assembly = parse_docx_chunks_from_bytes(bytes, path.as_deref(), owner_id)?;
                assembly.document.file_name = file_name;
                Ok(ParsedSourceFile::Docx(assembly))
            }
        }
    }

    fn from_parts(
        file_name: String,
        bytes: Vec<u8>,
        path: Option<PathBuf>,
    ) -> Result<Self, String> {
        if bytes.is_empty() {
            return Err("file is empty".to_string());
        }
        let extension = lowercase_extension(Path::new(&file_name))
            .ok_or_else(|| "invalid file format".to_string())?;

        match extension.as_str() {
            "pdf" => Ok(Self::Pdf {
                bytes,
                path,
                file_name,
            }),
            "docx" => Ok(Self::Docx {
                bytes,
                path,
                file_name,
            }),
            _ => Err("invalid file format".to_string()),
        }
    }
}

/// Streams a local file through SHA-256 without loading it into memory.
pub(crate) fn sha256_file(path: &Path, kind: &str) -> Result<String, String> {
    let mut file = File::open(path).map_err(|err| {
        format!(
            "failed to open {kind} for hashing {}: {err}",
            path.display()
        )
    })?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let bytes_read = file
            .read(&mut buffer)
            .map_err(|err| format!("failed to hash {kind} {}: {err}", path.display()))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Resolves an optional parser path to its canonical form when possible.
pub(crate) fn canonical_source_path(path: Option<&Path>) -> Option<PathBuf> {
    path.map(|path| path.canonicalize().unwrap_or_else(|_| PathBuf::from(path)))
}

#[cfg(test)]
#[path = "../../../../tests/unit/domains/documents/formats/source_file.rs"]
mod tests;
