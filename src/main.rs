use std::env;

// Available if you need it!
// use serde_bencode

fn decode_bencoded_value(encoded_value: &str) -> Result<serde_json::Value, String> {
    if let Some((len, rest)) = encoded_value.split_once(':') {
        if let Ok(len) = len.parse::<usize>() {
            let value = rest[..len].to_string();
            return Ok(serde_json::Value::String(value));
        }
    }
    Err(format!("Unhandled encoded value: {}", encoded_value))
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        println!("Logs from your program will appear here!");

        // Uncomment this block to pass the first stage
        let encoded_value = &args[2];
        match decode_bencoded_value(encoded_value) {
            Ok(decoded_value) => println!("{}", decoded_value),
            Err(err) => eprintln!("error: {}", err),
        }
    } else {
        println!("unknown command: {}", args[1])
    }
}
