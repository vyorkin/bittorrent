use std::{net::SocketAddrV4, path::PathBuf};

use anyhow::Context;
use bittorrent::{
    bencode::decode_bencoded_value,
    peer::{Handshake, Message, MessageFramer, MessageTag, Piece, Request},
    torrent::{Keys, Torrent},
    tracker::{urlencode, TrackerRequest, TrackerResponse},
};
use clap::{Parser, Subcommand};
use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const BLOCK_MAX: usize = 1 << 14;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
#[clap(rename_all = "snake_case")]
enum Command {
    Decode {
        value: String,
    },
    Info {
        torrent: PathBuf,
    },
    Peers {
        torrent: PathBuf,
    },
    Handshake {
        torrent: PathBuf,
        peer: String,
    },
    DownloadPiece {
        #[arg(short)]
        output: PathBuf,
        torrent: PathBuf,
        piece: usize,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Command::Decode { value } => {
            let (decoded_value, _) = decode_bencoded_value(&value);
            // let decoded_value: serde_json::Value = serde_bencode::from_str(&value).unwrap();
            println!("{}", decoded_value);
        }
        Command::Info { torrent } => {
            let torrent = Torrent::from_file(torrent).context("open torrent file")?;
            println!("Tracker URL: {}", torrent.announce);
            if let Keys::SingleFile { length } = torrent.info.keys {
                println!("Length: {}", length);
            } else {
                todo!();
            }

            println!("Info Hash: {}", hex::encode(torrent.info_hash()));
            println!("Piece Length: {}", torrent.info.piece_length);
            println!("Piece Hashes:");
            for hash in torrent.info.pieces.0 {
                println!("{}", hex::encode(hash));
            }
        }
        Command::Peers { torrent } => {
            let torrent = Torrent::from_file(torrent).context("open torrent file")?;
            let length = if let Keys::SingleFile { length } = torrent.info.keys {
                println!("Length: {}", length);
                length
            } else {
                todo!();
            };

            let request = TrackerRequest {
                peer_id: "00112233445566778899".into(),
                port: 6881,
                uploaded: 0,
                downloaded: 0,
                left: length,
                compact: 1,
            };

            // Info hash of the torrent, 20 bytes long, URL encoded.
            let info_hash = torrent.info_hash();

            let url_params =
                serde_urlencoded::to_string(&request).context("url-encode tracker parameters")?;
            let tracker_url = format!(
                "{}?{}&info_hash={}",
                torrent.announce,
                url_params,
                &urlencode(&info_hash)
            );
            let response = reqwest::get(tracker_url).await.context("query tracker")?;
            let response = response.bytes().await.context("fetch tracker response")?;
            let response: TrackerResponse =
                serde_bencode::from_bytes(&response).context("parse tracker response")?;

            for peer in response.peers.0 {
                println!("{}:{}", peer.ip(), peer.port());
            }
        }
        Command::Handshake { torrent, peer } => {
            let torrent = Torrent::from_file(torrent).context("open torrent file")?;
            let info_hash = torrent.info_hash();

            let peer = peer.parse::<SocketAddrV4>().context("parse peer address")?;
            let mut peer = tokio::net::TcpStream::connect(peer)
                .await
                .context("connect to peer")?;

            let mut handshake = Handshake::new(info_hash, *b"00112233445566778899");
            let handshake_bytes = handshake.as_bytes_mut();

            peer.write_all(handshake_bytes)
                .await
                .context("write handshake")?;

            peer.read_exact(handshake_bytes)
                .await
                .context("read handshake")?;

            assert_eq!(handshake.length, 19);
            assert_eq!(&handshake.bittorrent, b"BitTorrent protocol");

            println!("Peer ID: {}", hex::encode(handshake.peer_id));
        }
        Command::DownloadPiece {
            output,
            torrent,
            piece,
        } => {
            let torrent = Torrent::from_file(torrent).context("open torrent file")?;

            assert!(piece < torrent.info.pieces.0.len());

            let length = if let Keys::SingleFile { length } = torrent.info.keys {
                println!("Length: {}", length);
                length
            } else {
                todo!();
            };

            let request = TrackerRequest {
                peer_id: "00112233445566778899".into(),
                port: 6881,
                uploaded: 0,
                downloaded: 0,
                left: length,
                compact: 1,
            };

            // Info hash of the torrent, 20 bytes long, URL encoded.
            let info_hash = torrent.info_hash();

            let url_params =
                serde_urlencoded::to_string(&request).context("url-encode tracker parameters")?;
            let tracker_url = format!(
                "{}?{}&info_hash={}",
                torrent.announce,
                url_params,
                &urlencode(&info_hash)
            );
            let response = reqwest::get(tracker_url).await.context("query tracker")?;
            let response = response.bytes().await.context("fetch tracker response")?;
            let tracker_info: TrackerResponse =
                serde_bencode::from_bytes(&response).context("parse tracker response")?;

            let peer = tracker_info.peers.0[0];

            let mut peer = tokio::net::TcpStream::connect(peer)
                .await
                .context("connect to peer")?;

            let mut handshake = Handshake::new(info_hash, *b"00112233445566778899");
            let handshake_bytes = handshake.as_bytes_mut();

            peer.write_all(handshake_bytes)
                .await
                .context("write handshake")?;

            peer.read_exact(handshake_bytes)
                .await
                .context("read handshake")?;

            assert_eq!(handshake.length, 19);
            assert_eq!(&handshake.bittorrent, b"BitTorrent protocol");

            println!("Peer ID: {}", hex::encode(handshake.peer_id));

            let mut peer = tokio_util::codec::Framed::new(peer, MessageFramer {});
            let bitfield = peer
                .next()
                .await
                .expect("peer always sends bitfields")
                .context("peer message was invalid")?;

            assert_eq!(bitfield.tag, MessageTag::Bitfield);

            peer.send(Message {
                tag: MessageTag::Interested,
                payload: Vec::new(),
            })
            .await
            .context("send interested message")?;

            let unchoke = peer
                .next()
                .await
                .expect("peer always sends unchoke")
                .context("peer message was invalid")?;

            assert_eq!(unchoke.tag, MessageTag::Unchoke);
            assert!(unchoke.payload.is_empty());

            let piece_index = piece;
            let piece_hash = torrent.info.pieces.0[piece_index];
            let piece_size = if piece_index == torrent.info.pieces.0.len() + 1 {
                length % torrent.info.piece_length
            } else {
                torrent.info.piece_length
            };

            let mut all_blocks: Vec<u8> = Vec::with_capacity(piece_size);
            let num_blocks = (piece_size / (BLOCK_MAX - 1)) / BLOCK_MAX;
            for block in 0..num_blocks {
                let block_size = if block == num_blocks - 1 {
                    piece_size % BLOCK_MAX
                } else {
                    BLOCK_MAX
                };
                let mut request = Request::new(
                    piece_index as u32,
                    (block * BLOCK_MAX) as u32,
                    block_size as u32,
                );
                let request_bytes = Vec::from(request.as_bytes_mut());
                peer.send(Message {
                    tag: MessageTag::Request,
                    payload: request_bytes,
                })
                .await
                .with_context(|| format!("send request for block {block}"))?;

                let piece = peer
                    .next()
                    .await
                    .expect("peer always sends piece")
                    .context("peer message was invalid")?;

                assert_eq!(piece.tag, MessageTag::Piece);
                assert!(!piece.payload.is_empty());

                let piece = &(piece.payload[..]) as *const [u8] as *const Piece;
                let piece = unsafe { &*piece };

                all_blocks.extend(piece.block());
            }
        }
    }
    Ok(())
}
