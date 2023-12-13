use std::env;

// Available if you need it!
// use serde_bencode

fn decode_bencoded_value(encoded_value: &str) -> (serde_json::Value, &str) {
    let (tag, mut rest) = encoded_value.split_at(1);
    let tag_char = tag.chars().next().expect("Tag doesn't exist");
    match tag_char {
        'i' => {
            // i52e
            if let Some((s, remainder)) = rest.split_once('e') {
                if let Ok(n) = s.parse::<i64>() {
                    return (n.into(), remainder);
                }
            }
        }
        'l' => {
            // l<bencoded_elements>e
            // l5:helloi52ee
            let mut values = Vec::new();
            while !rest.is_empty() && !rest.starts_with('e') {
                let (v, remainder) = decode_bencoded_value(rest);
                rest = remainder;
                values.push(v);
            }
            return (values.into(), &rest[1..]);
        }
        '0'..='9' => {
            if let Some((str, rest)) = rest.split_once(':').and_then(|(_, remainder)| {
                tag.parse::<usize>()
                    .ok()
                    .map(|len| (remainder[..len].to_string(), &remainder[len..]))
            }) {
                return (str.into(), rest);
            }
        }
        _ => {}
    }

    panic!("unhandled encoded value: {}", encoded_value);
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let (decoded_value, _) = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value);
    } else {
        println!("unknown command: {}", args[1])
    }
}
