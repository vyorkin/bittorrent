use std::{net::SocketAddrV4, path::PathBuf};

use anyhow::Context;
use bittorrent::{
    bencode::decode_bencoded_value,
    peer::Handshake,
    torrent::{Keys, Torrent},
    tracker::{urlencode, TrackerRequest, TrackerResponse},
};
use clap::{Parser, Subcommand};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Decode { value: String },
    Info { torrent: PathBuf },
    Peers { torrent: PathBuf },
    Handshake { torrent: PathBuf, peer: String },
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

            let handshake_bytes =
                &mut handshake as *mut Handshake as *mut [u8; std::mem::size_of::<Handshake>()];
            let handshake_bytes: &mut [u8; std::mem::size_of::<Handshake>()] =
                unsafe { &mut *handshake_bytes };

            peer.write_all(handshake_bytes)
                .await
                .context("write handshake")?;

            peer.read_exact(handshake_bytes)
                .await
                .context("read handshake")?;

            assert_eq!(handshake.length, 19);
            assert_eq!(&handshake.bittorrent, b"BitTorrent protocol");

            println!("Peer ID: {}", hex::encode(&handshake.peer_id));
        }
    }
    Ok(())
}
