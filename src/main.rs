use std::env;

// Available if you need it!
// use serde_bencode

fn decode_bencoded_value(encoded_value: &str) -> Result<serde_json::Value, String> {
    if let Some(rest) = encoded_value.strip_prefix('i') {
        if let Some(end) = rest.find('e') {
            if let Ok(integer) = encoded_value[1..end + 1].parse::<i32>() {
                return Ok(serde_json::Value::Number(integer.into()));
            }
        }
    } else if let Some((len, rest)) = encoded_value.split_once(':') {
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
        let encoded_value = &args[2];
        match decode_bencoded_value(encoded_value) {
            Ok(decoded_value) => println!("{}", decoded_value),
            Err(err) => eprintln!("error: {}", err),
        }
    } else {
        println!("unknown command: {}", args[1])
    }
}
