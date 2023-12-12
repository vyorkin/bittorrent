use std::env;

// Available if you need it!
// use serde_bencode

fn decode_bencoded_value(encoded_value: &str) -> Result<serde_json::Value, String> {
    if let Some(n) = encoded_value
        .strip_prefix('i')
        .and_then(|rest| rest.strip_suffix('e'))
        .and_then(|s| s.parse::<i64>().ok())
    {
        return Ok(n.into());
    } else if let Some(len) = encoded_value
        .split_once(':')
        .and_then(|(len, rest)| len.parse::<usize>().ok().map(|len| rest[..len].to_string()))
    {
        return Ok(len.into());
    }
    Err(format!("Unhandled encoded value: {}", encoded_value))
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        match decode_bencoded_value(encoded_value) {
            Ok(decoded_value) => println!("{}", decoded_value),
            Err(err) => eprintln!("error: {}", err),
        }
    } else {
        println!("unknown command: {}", args[1])
    }
}
