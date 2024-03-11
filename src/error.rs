use thiserror::Error;

#[derive(Error, Debug)]
pub enum BittorrentError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error("Bencode error")]
    BencodeError(#[from] serde_bencode::Error),
}
