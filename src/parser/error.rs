use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("tracks.md not found at {0}")]
    IndexNotFound(PathBuf),

    #[error("Invalid metadata for track {track_id}: {message}")]
    MetadataInvalid { track_id: String, message: String },

    #[error("Failed to read {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("No tracks found in {0}")]
    EmptyIndex(PathBuf),
}
