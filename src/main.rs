use std::path::PathBuf;

use anyhow::Context;
use bittorrent::{
    bencode::decode_bencoded_value,
    torrent::{Keys, Torrent},
};
use clap::{Parser, Subcommand};
use sha1::{Digest, Sha1};

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
}

// Available if you need it!
// use serde_bencode

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Command::Decode { value } => {
            let (decoded_value, _) = decode_bencoded_value(&value);
            // let decoded_value: serde_json::Value = serde_bencode::from_str(&value).unwrap();
            println!("{}", decoded_value);
        }
        Command::Info { torrent } => {
            let dot_torrent = std::fs::read(torrent).context("open torrent file")?;
            let torrent: Torrent =
                serde_bencode::from_bytes(&dot_torrent).context("parse torrent file")?;
            println!("Tracker URL: {}", torrent.announce);
            if let Keys::SingleFile { length } = torrent.info.keys {
                println!("Length: {}", length);
            } else {
                todo!();
            }

            let info_encoded =
                serde_bencode::to_bytes(&torrent.info).context("re-encode info section")?;
            let info_hash = Sha1::digest(info_encoded);
            println!("Info Hash: {}", hex::encode(info_hash));
            println!("Piece Length: {}", torrent.info.piece_length);
            println!("Piece Hashes:");
            for hash in torrent.info.pieces.0 {
                println!("{}", hex::encode(hash));
            }
        }
    }
    Ok(())
}
