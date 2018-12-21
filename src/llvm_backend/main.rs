mod parser;
use parser::Parser;

fn main() {
    println!("= Parsing wasm code! =");
    let code: Vec<u8> = vec![
        0x00, 0x61, 0x73, 0x6d, // WASM_BINARY_MAGIC
        0x01, 0x00, 0x00, 0x00, // WASM_BINARY_VERSION
        
    ];
    Parser::new(&code).module();
}
