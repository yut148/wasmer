mod parser;
use parser::Parser;

fn main() {
    let preamble: Vec<u8> = vec![
        0x00, 0x61, 0x73, 0x6d, // WASM_BINARY_MAGIC
        0x01, 0x00, 0x00, 0x00, // WASM_BINARY_VERSION
        0x01,
    ];

    let type_section: Vec<u8> = vec![
        0x00, 0x61, 0x73, 0x6d, // WASM_BINARY_MAGIC
        0x01, 0x00, 0x00, 0x00, // WASM_BINARY_VERSION
        0x01, // SECTION ID: TYPE
        0x06, // PAYLOAD LEN
        0x01, // COUNT
        0x60, // FORM: FUNC_TYPE
        0x01, // PARAM COUNT
        0x7f, // PARAM TYPE
        0x01, // RETURN COUNT
        0x7f, // RETURN TYPE
    ];

    // let import_section: Vec<u8> = vec![
    //     0x00, 0x61, 0x73, 0x6d, // WASM_BINARY_MAGIC
    //     0x01, 0x00, 0x00, 0x00, // WASM_BINARY_VERSION
    //     0x01, // SECTION ID: TYPE
    //     0x06, // PAYLOAD LEN
    //     0x01, // COUNT
    //     0x60, // FORM: FUNC_TYPE
    //     0x01, // PARAM COUNT
    //     0x7f, // PARAM TYPE
    //     0x01, // RETURN COUNT
    //     0x7f, // RETURN TYPE
    // ];

    Parser::new(&func_type).module();
}
