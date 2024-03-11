use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

/// Metainfo files (also known as .torrent files)
#[derive(Debug, Clone, Deserialize)]
pub struct Torrent {
    /// The URL of the tracker.
    pub announce: String,
    /// Torrent
    pub info: Info,
}

impl Torrent {
    pub fn info_hash(&self) -> [u8; 20] {
        self.info.hash()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Info {
    /// Suggested name to save file, purely advisory.
    /// UTF-8 encoded string.
    /// In the single file case, it is the name of a file.
    /// In a multiple file case, it is the name of a directory.
    pub name: String,
    /// Number of bytes in each piece the file is split into.
    #[serde(rename = "piece length")]
    pub piece_length: usize,
    /// Each entry of pieces is the SHA1 hash of the corresponding index.
    pub pieces: hashes::Hashes,
    /// Download represents a single file or a set of files.
    #[serde(flatten)]
    pub keys: Keys,
}

impl Info {
    pub fn hash(&self) -> [u8; 20] {
        let encoded = serde_bencode::to_bytes(&self).expect("bencode should be fine");
        Sha1::digest(encoded).into()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Keys {
    /// If `length` is present then the download represents a single file.
    SingleFile {
        /// In the single file case the `length` maps to the length of the file in bytes.
        length: usize,
    },
    /// Otherwise it represents a set of files which go in a directory structure.
    /// For the purposes of the other keys in `Info`, the multi-file case is treated as
    /// only having a single file by concatenating the files in the order they appear in the files list.
    MultiFile { files: Vec<File> },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct File {
    /// The length of the file in bytes.
    pub length: usize,
    /// Subdirectory names for this file, the last of which is the actual file name.
    pub path: Vec<String>,
}

mod hashes {
    use serde::de::{self, Visitor};
    use serde::{Deserialize, Serialize, Serializer};

    #[derive(Debug, Clone)]
    pub struct Hashes(pub Vec<[u8; 20]>);
    struct HashesVisitor;

    impl<'de> Visitor<'de> for HashesVisitor {
        type Value = Hashes;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a byte string whose length is a multiple of 20")
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if v.len() % 20 != 0 {
                return Err(E::custom(format!("length is {}", v.len())));
            }
            let chunks = v
                .chunks_exact(20)
                .map(|slice_20| slice_20.try_into().expect("guaranteed to be length 20"))
                .collect();

            Ok(Hashes(chunks))
        }
    }

    impl<'de> Deserialize<'de> for Hashes {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_bytes(HashesVisitor)
        }
    }

    impl Serialize for Hashes {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let single_slice = self.0.concat();
            serializer.serialize_bytes(&single_slice)
        }
    }
}
