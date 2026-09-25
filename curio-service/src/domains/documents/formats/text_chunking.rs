use std::sync::OnceLock;

use tiktoken_rs::{CoreBPE, o200k_base};

pub const MAX_TOKEN_CHUNK: usize = 800;

/// First probe window when searching for a chunk end. Typical prose is a few
/// bytes per token, so a chunk usually fits well inside this window.
const INITIAL_PROBE_BYTES: usize = 8 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenChunkRange {
    pub start_offset: usize,
    pub end_offset: usize,
    pub token_count: usize,
}

pub(crate) fn tokenizer() -> &'static CoreBPE {
    static TOKENIZER: OnceLock<CoreBPE> = OnceLock::new();
    TOKENIZER.get_or_init(|| o200k_base().expect("o200k tokenizer data should be valid"))
}

/// Returns contiguous token-bounded ranges with exclusive UTF-8 byte offsets.
///
/// Each range is the longest prefix of the remaining text whose token count
/// does not exceed [`MAX_TOKEN_CHUNK`]. The search window grows geometrically
/// before bisecting, so each step tokenizes text proportional to one chunk
/// rather than the whole remaining document.
pub fn token_bounded_ranges(text: &str) -> Vec<TokenChunkRange> {
    if text.is_empty() {
        return Vec::new();
    }

    let tokenizer = tokenizer();
    let count = |start: usize, end: usize| tokenizer.count_with_special_tokens(&text[start..end]);
    let boundaries = text
        .char_indices()
        .map(|(offset, _)| offset)
        .chain(std::iter::once(text.len()))
        .collect::<Vec<_>>();
    let last_index = boundaries.len() - 1;
    let mut ranges = Vec::new();
    let mut start_index = 0;

    while start_index < last_index {
        let start_offset = boundaries[start_index];

        // Gallop: find an index whose prefix exceeds the budget, or the end.
        let mut low = start_index + 1;
        let mut high = last_index;
        let mut probe_bytes = INITIAL_PROBE_BYTES;
        loop {
            let probe_offset = start_offset.saturating_add(probe_bytes);
            if probe_offset >= text.len() {
                break;
            }
            let probe_index = boundaries.partition_point(|&offset| offset < probe_offset);
            if count(start_offset, boundaries[probe_index]) > MAX_TOKEN_CHUNK {
                high = probe_index;
                break;
            }
            low = probe_index;
            probe_bytes = probe_bytes.saturating_mul(2);
        }

        // Bisect for the longest in-budget prefix within (low, high]. `low` is
        // either a single character or a probe already known to fit.
        let mut best_end_index = low;
        let mut best_token_count = count(start_offset, boundaries[best_end_index]);
        let mut search_low = best_end_index + 1;
        let mut search_high = high;
        while search_low <= search_high {
            let midpoint = search_low + (search_high - search_low) / 2;
            let token_count = count(start_offset, boundaries[midpoint]);
            if token_count <= MAX_TOKEN_CHUNK {
                best_end_index = midpoint;
                best_token_count = token_count;
                search_low = midpoint + 1;
            } else {
                search_high = midpoint - 1;
            }
        }

        ranges.push(TokenChunkRange {
            start_offset,
            end_offset: boundaries[best_end_index],
            token_count: best_token_count,
        });
        start_index = best_end_index;
    }

    ranges
}

#[cfg(test)]
#[path = "../../../../tests/unit/domains/documents/formats/text_chunking.rs"]
mod tests;
