use serde::{Deserialize, Serialize};

use self::peers::Peers;

#[derive(Debug, Clone, Serialize)]
pub struct TrackerRequest {
    /// Client unique identifier, a string of length 20.
    pub peer_id: String,
    /// Port client is listening on.
    pub port: u16,
    /// Total amount uploaded so far.
    pub uploaded: usize,
    /// Total amount downloaded so far.
    pub downloaded: usize,
    /// Number of bytes left to download.
    pub left: usize,
    /// Whether the peer list should use compact representation.
    pub compact: u8,
}

pub fn urlencode(t: &[u8; 20]) -> String {
    let mut encoded = String::with_capacity(3 * t.len());
    for &byte in t {
        encoded.push('%');
        encoded.push_str(&hex::encode([byte]));
    }
    encoded
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrackerResponse {
    /// Indicates how often a client should make requests to the tracker (in seconds).
    pub interval: usize,
    /// Contains a list of peers that client can connect to. Each peer is represented using 6 bytes.
    /// The first 4 bytes are the peer's IP address and the last 2 bytes are the peer's port number.
    pub peers: Peers,
}

mod peers {
    use std::net::{Ipv4Addr, SocketAddrV4};

    use serde::de::{self, Visitor};
    use serde::{Deserialize, Serialize, Serializer};

    #[derive(Debug, Clone)]
    pub struct Peers(pub Vec<SocketAddrV4>);
    struct PeersVisitor;

    impl<'de> Visitor<'de> for PeersVisitor {
        type Value = Peers;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str(
                "6 bytes, first 4 bytes are the peer's IP address and the last 2 bytes are the peer's port number",
            )
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if v.len() % 6 != 0 {
                return Err(E::custom(format!("length is {}", v.len())));
            }

            let chunks = v
                .chunks_exact(6)
                .map(|slice_6| {
                    SocketAddrV4::new(
                        Ipv4Addr::new(slice_6[0], slice_6[1], slice_6[2], slice_6[3]),
                        u16::from_be_bytes([slice_6[4], slice_6[5]]),
                    )
                })
                .collect();

            Ok(Peers(chunks))
        }
    }

    impl<'de> Deserialize<'de> for Peers {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_bytes(PeersVisitor)
        }
    }

    impl Serialize for Peers {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut single_slice = Vec::with_capacity(6 * self.0.len());
            for peer in &self.0 {
                single_slice.extend(peer.ip().octets());
                single_slice.extend(peer.port().to_be_bytes());
            }
            serializer.serialize_bytes(&single_slice)
        }
    }
}
