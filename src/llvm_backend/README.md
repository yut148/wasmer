
# WASMER LLVM
### NOT CURRENTLY SUPPORTED
- wasm64
- Multiple wasm instances per process. Easy to refactor.
- Multiple linear memories per wasm instance. Easy to refactor.

### ASSUMPTIONS
- Expects a Context type to have the following structure
- Expects memory storage strategy to take advantage of guard pages, so it does no bounds checking.

### GOAL
- Be faster than the Cranelift backend (compile-time and runtime)
- Have an API similar to cranelift-wasm

### STRATEGY
- Single-pass parsing and codegen from wasm to LLVM IR

### CURRENT SUPPORT
- preamble
- types
- imports
- functions

### TODO
- wasm32 to LLVMIR
- fuzz tests
- unittests
- validation (utf8 [Unicode Standard 11.0, Section 3.9, Table 3-7. Well-Formed UTF-8 Byte Sequences] and semantics)
https://www.unicode.org/versions/Unicode11.0.0/ch03.pdf
https://webassembly.github.io/spec/core/binary/values.html#names
- error messages and making error position point of error instead of start_position

### FUTURE ADDITIONS
- Lazy compilation

### PROCESS ADDRESS SPACE (LINUX x86-64 EXAMPLE)
```
+++++++++++++++++++++++++++
|         STACK
+++++++++++++++++++++++++++
|    SHARED LIBRARIES
+++++++++++++++++++++++++++
|          HEAP
|
|
|  ++++++ INSTANCE ++++++
|  ++++++++++++++++++++++
|  |                    |
|  |    LINEAR MEMORY   | RW
|  |                    |
|  ++++++++++++++++++++++
|            :
|  ++++++++++++++++++++++
|  |      GLOBALS       | R
|  ++++++++++++++++++++++
|            :
|  ++++++++++++++++++++++
|  |       TABLES       | R
|  ++++++++++++++++++++++
|            :
|  ++++++++++++++++++++++
|  |        CODE        | E
|  ++++++++++++++++++++++
|
+++++++++++++++++++++++++++
|      BSS SEGMENT
+++++++++++++++++++++++++++
|    (RO)DATA SEGMENT
+++++++++++++++++++++++++++
|      TEXT SEGMENT
+++++++++++++++++++++++++++
```

### WASM ENCODING
```
+++++++++++++++++++++++++++
|         MARK
+++++++++++++++++++++++++++
|         TYPES
+++++++++++++++++++++++++++
|        IMPORTS
+++++++++++++++++++++++++++
...
```

### NOTES
- The instantiate function is what sepeartes the different backends
-


### ROUGH PIPELINE
```rust
// This is the part that seperates the various backend
fn instantiate(source: Vec<u8>, imports: Import) -> (Module, Instance) {
    let module: Module = wasm::compile(source); // IR & info
    let instance: Instance::new(&module, imports, options); // Compiled & relocated code
    (module, instance)
}

fn start() {
    let source: Vec<u8> = wasm::to_buf("./file.wasm");
    let (module, instance) = instantiate(source, imports);
    let func: u32 = module.exports.get_function_index("main"); // Exported function
    let signature: Vec<u8> = module.get_signature(func);
    if !compare!(signature, i32, i32, i32) { panic!("signature mismatch"); }
    let main: fn (u32, u32) -> u32 = instance.get_function_addr(func);
    let ctx: Context = instance.generate_context();
    main(0, 0, &ctx);
}

struct Module {
    pub signatures: Vec<Vec<u8>>, // [(0, 0), (1, 1, 0), ...]
    pub imports: Imports {
        pub functions: HashMap<(String, String), u32>,
    },
    pub exports: Exports {
        pub functions: HashMap<String, u32>,
    },
    pub functions: HashMap<String, u32>,
    ...
    pub module: LLVMModule,
}

impl Module {
    fn compile_and_link(imported_functions: Vec<*const u8>) -> Vec<*const u8>;
    fn get_import_names() -> Vec<(String, String)>;
}

struct Instance {
    tables: Vec<Table>, // Tables are always bounds checked
    memories: Vec<Memory>,
    globals: Vec<i64>,
    functions: Vec<*const u8>, // For internally-defined functions. They hold pointers to leaked Vec<const u8>
}

impl Instance {
    fn new(module: &mut Module, imports: Imports, options: InstanceOptions) -> Self {
        // NOTE: functions, globals, tables, memories must be stored in the order they appear in the module
        ...
        for (module, field) in module.get_import_names() {
            match imports.get((module, field)) {
                ImportValue::Func(f) => imported_functions.push(functions),
            }
        }

        let functions: Vec<*const u8> = module.compile_and_link(imported_functions); // Compile and link all function IRs
        ...

        Instance {
            functions,
            ...
        }
    }

    fn generate_context() -> Context {
        Context {
        }
    }
}

struct Context {
    pub memories: UncheckedSlice<UncheckedSlice<u8>>,
    pub tables: UncheckedSlice<BoundedSlice<u32>>,
    pub globals: UncheckedSlice<u8>,
}

struct Memory {
    pub base_ptr: *const u8,
    pub current: u32,
    pub maximum: u32
}

struct Table {
    pub base_ptr: *const u8,
    pub length: u32,
}

struct Imports {
    map: HashMap<(String, String), ImportValue>
}

enum ImportValue {
    Func(*const u8),
    ...
}
```

### REFACTOR AFTER PROTOTYPE
- Restructure project
    - errors
        - error.rs
            * wasmer::wasm::Error
            * wasmer::wasm::error_to_string
    - parser
        - parser.rs
            * wasmer::wasm::Parser
    - examples
        - malformed
            - bytes
            - wasm
            - wat
        - well_formed
            - bytes
            - wasm
            - wat
    - semantics
        - validate.rs
            * wasmer::wasm::validate_utf8
    - jit
        - module.rs
            * wasmer::jit::Module
        - instance.rs
            * wasmer::jit::Instance
        - memory
            * wasmer::jit::Memory
        ...

### TO MAKE IT WASM64
- import and member module index must be u64
- tables should have a boundedSlice of u64
- indexing memories, tables, globals should be in u64

# ENCODING
## LEB128
#### UNSIGNED
```
MSB ------------------ LSB
00001001 10000111 01100101  In raw binary
 0100110  0001110  1100101  Padded to a multiple of 7 bits
00100110 10001110 11100101  Add high 1 bits on all but last (most significant) group to form bytes

Starting with the first byte, get rid of the msb bit
11100101 -> 11001010 << 1
11001010 -> 01100101 >> 1

Widen to appropriate type
01100101 -> 00000000 01100101 as u32

For the next byte, get rid of the msb bit
10001110 -> 00011100 << 1
00011100 -> 00001110 >> 1

Widen to appropriate type
00001110 -> 00000000 00001110 as u32

Shift by 7 bits to the left
00000000 00001110 -> 00000111 00000000 << 7

Or the value with the previous result
00000111 00000000 | 00000000 01100101

And so on. Basically you shift by a multiple of 7

if byte's msb is unset, you can break the loop
```
#### SIGNED
```
MSB ------------------ LSB
00000110 01111000 10011011  In raw two's complement binary
 1011001  1110001  0011011  Sign extended to a multiple of 7 bits
01011001 11110001 10011011  Add high 1 bits on all but last (most significant) group to form bytes

Starting with the first byte, get rid of the msb bit
10011011 -> 00110110 << 1
00110110 -> 00011011 >> 1

Widen to appropriate type
01100101 -> 00000000 01100101 as u32

For the next byte, get rid of the msb bit
10001110 -> 00011100 << 1
00011100 -> 00001110 >> 1

Widen to appropriate type
00001110 -> 00000000 00001110 as u32

Shift by 7 bits to the left
00000000 00001110 -> 00000111 00000000 << 7

Or the value with the previous result
00000111 00000000 | 00000000 01100101

And so on. Basically you shift by a multiple of 7

Decoding a signed LEB128 encoding has an extra twist, we need to extend the sign bit

If the value is signed, then the msb is set

if result & 0x0100_0000 == 0x0100_0000 {
    result |= !(0x1 << encoding_size)
}

if byte's msb is unset, you can break the loop
```

### WELL-FORMED UTF-8 BYTE SEQUENCES
Based on Unicode Standard 11.0, Section 3.9, Table 3-7.

| Code Points        | First Byte   | Second Byte    | Third Byte    | Fourth Byte   |
|:-------------------|:-------------|:---------------|:--------------|:--------------|
| U+0000..U+007F     | 00..7F       |                |               |               |
| U+0080..U+07FF     | C2..DF       | 80..BF         |               |               |
| U+0800..U+0FFF     | E0           | A0..BF         | 80..BF        |               |
| U+1000..U+CFFF     | E1..EC       | 80..BF         | 80..BF        |               |
| U+D000..U+D7FF     | ED           | 80..9F         | 80..BF        |               |
| U+E000..U+FFFF     | EE..EF       | 80..BF         | 80..BF        |               |
| U+10000..U+3FFFF   | F0           | 90..BF         | 80..BF        | 80..BF        |
| U+40000..U+FFFFF   | F1..F3       | 80..BF         | 80..BF        | 80..BF        |
| U+100000..U+10FFFF | F4           | 80..8F         | 80..BF        | 80..BF        |

