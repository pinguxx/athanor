//! Recursive character text splitter for chunking documents.

use crate::document::Document;
use crate::error::RagError;

/// Splits text into overlapping chunks using a recursive delimiter strategy.
///
/// Delimiters are evaluated in order of semantic preservation:
/// paragraphs (`\n\n`), newlines (`\n`), sentence boundaries (`. `), words (` `), and characters.
#[derive(Clone, Debug)]
pub struct RecursiveCharacterSplitter {
    chunk_size: usize,
    chunk_overlap: usize,
    separators: Vec<String>,
}

impl Default for RecursiveCharacterSplitter {
    fn default() -> Self {
        Self {
            chunk_size: 500,
            chunk_overlap: 50,
            separators: vec![
                "\n\n".to_string(),
                "\n".to_string(),
                ". ".to_string(),
                " ".to_string(),
                "".to_string(),
            ],
        }
    }
}

impl RecursiveCharacterSplitter {
    /// Construct a new text splitter with the specified chunk size and overlap.
    ///
    /// # Errors
    /// Returns `RagError::Config` if `chunk_size == 0` or if `chunk_overlap >= chunk_size`.
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Result<Self, RagError> {
        if chunk_size == 0 {
            return Err(RagError::Config(
                "chunk_size must be greater than zero".into(),
            ));
        }
        if chunk_overlap >= chunk_size {
            return Err(RagError::Config(
                "chunk_overlap must be strictly less than chunk_size".into(),
            ));
        }

        Ok(Self {
            chunk_size,
            chunk_overlap,
            separators: vec![
                "\n\n".to_string(),
                "\n".to_string(),
                ". ".to_string(),
                " ".to_string(),
                "".to_string(),
            ],
        })
    }

    /// Set custom delimiters in order of priority.
    pub fn with_separators(mut self, separators: Vec<String>) -> Self {
        self.separators = separators;
        self
    }

    /// Split a raw string slice into overlapping chunks.
    pub fn split_text(&self, text: &str) -> Vec<String> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Vec::new();
        }

        if trimmed.len() <= self.chunk_size {
            return vec![trimmed.to_string()];
        }

        self.split_recursive(trimmed, &self.separators)
    }

    fn split_recursive(&self, text: &str, separators: &[String]) -> Vec<String> {
        if text.len() <= self.chunk_size {
            return vec![text.to_string()];
        }

        // Find the highest-priority separator present in the text
        let mut chosen_sep = None;
        let mut next_seps = &separators[separators.len()..];

        for (i, sep) in separators.iter().enumerate() {
            if sep.is_empty() || text.contains(sep.as_str()) {
                chosen_sep = Some(sep.as_str());
                next_seps = &separators[i + 1..];
                break;
            }
        }

        let sep = chosen_sep.unwrap_or_default();

        // Split text by the chosen separator
        let raw_splits: Vec<&str> = if sep.is_empty() {
            // Split by characters
            text.split("").filter(|s| !s.is_empty()).collect()
        } else {
            text.split(sep).collect()
        };

        let mut chunks: Vec<String> = Vec::new();
        let mut current_pieces: Vec<String> = Vec::new();
        let mut current_length = 0;

        for piece in raw_splits {
            let piece_str = piece.trim();
            if piece_str.is_empty() {
                continue;
            }

            // If a single piece exceeds chunk_size, split it with finer delimiters
            if piece_str.len() > self.chunk_size && !next_seps.is_empty() {
                // Flush current pieces first
                if !current_pieces.is_empty() {
                    chunks.push(current_pieces.join(sep));
                    current_pieces.clear();
                    current_length = 0;
                }

                let sub_chunks = self.split_recursive(piece_str, next_seps);
                chunks.extend(sub_chunks);
                continue;
            }

            let separator_len = if current_pieces.is_empty() {
                0
            } else {
                sep.len()
            };
            if current_length + separator_len + piece_str.len() > self.chunk_size
                && !current_pieces.is_empty()
            {
                let chunk_text = current_pieces.join(sep);
                chunks.push(chunk_text);

                // Retain overlapping pieces from the end of current_pieces
                let mut overlap_pieces: Vec<String> = Vec::new();
                let mut overlap_len = 0;

                for p in current_pieces.into_iter().rev() {
                    let sep_extra = if overlap_pieces.is_empty() {
                        0
                    } else {
                        sep.len()
                    };
                    if overlap_len + sep_extra + p.len() <= self.chunk_overlap {
                        overlap_len += sep_extra + p.len();
                        overlap_pieces.push(p);
                    } else {
                        break;
                    }
                }

                overlap_pieces.reverse();
                current_pieces = overlap_pieces;
                current_length = overlap_len;
            }

            current_pieces.push(piece_str.to_string());
            current_length += if current_pieces.len() == 1 {
                piece_str.len()
            } else {
                sep.len() + piece_str.len()
            };
        }

        if !current_pieces.is_empty() {
            let chunk_text = current_pieces.join(sep);
            chunks.push(chunk_text);
        }

        chunks
    }

    /// Split a collection of documents into smaller chunk documents.
    ///
    /// Preserves parent metadata and attaches `parent_id` and `chunk_index` fields.
    pub fn split_documents(&self, docs: &[Document]) -> Vec<Document> {
        let mut result = Vec::new();

        for doc in docs {
            let chunks = self.split_text(&doc.content);
            if chunks.len() <= 1 {
                result.push(doc.clone());
            } else {
                for (i, chunk) in chunks.into_iter().enumerate() {
                    let mut meta = doc.metadata.clone();
                    if let serde_json::Value::Object(ref mut map) = meta {
                        map.insert("parent_id".to_string(), serde_json::json!(doc.id));
                        map.insert("chunk_index".to_string(), serde_json::json!(i));
                    }

                    result.push(Document {
                        id: format!("{}_{i}", doc.id),
                        content: chunk,
                        metadata: meta,
                    });
                }
            }
        }

        result
    }
}
