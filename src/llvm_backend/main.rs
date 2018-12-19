mod parser;
use parser::Parser;

fn main() {
    println!("= Parsing wasm code! =");
    let code: Vec<u8> = vec![
        0x00, 0x61, 0x73, 0x6d,
        0x01, 0x00, 0x00, 0x00,
    ];
    Parser::new(&code).module();
}
