mod parser;
use parser::Parser;

fn main() {
    println!("= Parsing wasm code! =");
    let code: Vec<u8> = vec![
        0x00, 0x61, 0x73, 0x6d,
        0x01, 0x00, 0x00, 0x00,
        0b01111111,
        // 0x7f,
    ];
    Parser::new(&code).module();
}

// 01111111
// 00000000 11111111
// 111111
