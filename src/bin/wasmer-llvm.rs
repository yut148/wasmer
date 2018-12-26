extern crate wasmer;

use wasmer::llvm_backend::parser::Parser;

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
        0x06, //   PAYLOAD LEN
        0x01, //   COUNT
        0x60, //   FORM: FUNC_TYPE
        0x01, //     PARAM COUNT
        0x7f, //     PARAM TYPE
        0x01, //     RETURN COUNT
        0x7f, //     RETURN TYPE
    ];

    let import_section: Vec<u8> = vec![
        0x00, 0x61, 0x73, 0x6d, // WASM_BINARY_MAGIC
        0x01, 0x00, 0x00, 0x00, // WASM_BINARY_VERSION
        0x01, // SECTION ID: TYPE
        0x07, //   PAYLOAD LEN: 7
        0x01, //   ENTRY COUNT: 1
        0x60, //   FORM: FUNC_TYPE
        0x02, //     PARAM COUNT: 2
        0x7f, //     PARAM TYPE: i32
        0x7e, //     PARAM TYPE: i64
        0x01, //     RETURN COUNT: 1
        0x7f, //     RETURN TYPE: i32
        0x02, // SECTION ID: IMPORT
        0x1b, //   PAYLOAD LEN:
        0x02, //   ENTRY COUNT: 2
        0x03, //   MODULE LEN: 3
        0x65, 0x6e, 0x76, // MODULE_STR: "env"
        0x04, //   FIELD LEN: 4
        0x66, 0x75, 0x6e, 0x63, // FIELD_STR: "func"
        0x00, //   EXTERNAL KIND: FUNCTION
        0x00, //     TYPE INDEX: 0
        0x03, //   MODULE LEN: 3
        0x65, 0x6e, 0x76, // MODULE_STR: "env"
        0x05, //   FIELD LEN: 5
        0x74, 0x61, 0x62, 0x6c, 0x65, // FIELD_STR: "table"
        0x01, //   EXTERNAL KIND: TABLE
        0x70, //     ELEMENT TYPE: any_func
        0x01, //     FLAGS: has_maximum
        0x00, //     INITIAL: 0
        0x01  //     MAXIMUM: 1
    ];

    Parser::new(&import_section).module();
}
