use std::collections::HashMap;

pub struct Parser<'a> {
    code: &'a Vec<u8>,
    cursor: usize,
    module: Module,
}

// TODO: Move
pub struct Module {
    pub signatures: Vec<Vec<u8>>,
    // pub imports: Imports,
    // pub exports: Exports,
    // pub functions: HashMap<String, u32>,
    // pub ir: LLVMModule,
}

impl Module {
    pub fn new() -> Self {
        Module {
            signatures: Vec::new(),
        }
    }
}


struct Exports {
    pub functions: HashMap<String, u32>,
}

struct Imports {
    pub functions: HashMap<(String, String), u32>,
}

impl <'a> Parser<'a> {
    pub fn new(code: &'a Vec<u8>) -> Self {
        Parser {
            code,
            cursor: 0,
            module: Module::new(),
        }
    }

    pub fn module(&mut self) {
        println!("= Parsing wasm module! =");
        // println!("char = {:?}", self.code[self.cusror]);
        self.preamble();
    }

    pub fn preamble(&mut self) -> bool {
        let magic_no = match self.unint32() {
            Some(v) => v,
            None => return false,
        };
        let version_no = match self.unint32() {
            Some(v) => v,
            None => return false,
        };
        println!("magic no = {:08x}", magic_no);
        println!("version no = {:08x}", version_no);
        true
    }

    fn eat_bytes(&mut self, range: usize) -> Option<&[u8]> {
        let start = self.cursor;
        let end = start + range;

        if end > self.code.len() {
            return None;
        }

        // Advance the cursor
        self.cursor = end;

        return Some(&self.code[start..end]);
    }

    fn eat_byte(&mut self) -> Option<u8> {
        let index = self.cursor;

        if index < self.code.len() {
            // Advance the cursor
            self.cursor += 1;
            return Some(self.code[index]);
        }
        None
    }

    /// Little-endian parsing
    pub fn unint32(&mut self) -> Option<u32> {
        let mut result = 0;

        if let Some(bytes) = self.eat_bytes(4) {
            let mut shift = 0;
            let mut result = 0;
            for byte in bytes {
                result |= (*byte as u32) << shift;
                shift += 8;
            }
            return Some(result);
        }
        None
    }



    // fn signature(&mut self) {
    // }

    // fn varuint1(&mut self) -> u8 {
    //     let mut result = 0;
    //     let mut shift = 0;

    //     for byte in bytes {
    //         // Unsetting the msb
    //         let value = (byte & !0b10000000) as u32;
    //         let value = value << shift;
    //         result |= value;
    //         if byte & 0b10000000 == 0  {
    //             break;
    //         }
    //         shift += 7;
    //     }
    //     result
    // }

    // fn varuint7(&mut self) -> u8 {
    //     let mut result = 0;
    //     let mut shift = 0;

    //     for byte in bytes {
    //         // Unsetting the msb
    //         let value = (byte & !0b10000000) as u32;
    //         let value = value << shift;
    //         result |= value;
    //         if byte & 0b10000000 == 0  {
    //             break;
    //         }
    //         shift += 7;
    //     }
    //     result
    // }

    // fn varuint32(&mut self) -> u32 {
    //     let mut result = 0;
    //     let mut shift = 0;

    //     for byte in bytes {
    //         // Unsetting the msb
    //         let value = (byte & !0b10000000) as u32;
    //         let value = value << shift;
    //         result |= value;
    //         if byte & 0b10000000 == 0  {
    //             break;
    //         }
    //         shift += 7;
    //     }
    //     result
    // }

    // fn varint1(&mut self) -> i8 {
    //     const size = 1;
    //     let mut result = 0;
    //     let mut shift = 0;

    //     for byte in bytes {
    //         // Unsetting the msb
    //         let value = (byte & !0b10000000) as u32;
    //         let value = value << shift;
    //         result |= value;
    //         if byte & 0b10000000 == 0  {
    //             break;
    //         }
    //         shift += 7;
    //     }

    //     // Unsetting the added sign bits
    //     result &= !(0xff_ff_ff_ff << size);
    //     result
    // }

    // fn varint7(&mut self) -> i8 {
    //     const size = 7;
    //     let mut result = 0;
    //     let mut shift = 0;

    //     for byte in bytes {
    //         // Unsetting the msb
    //         let value = (byte & !0b10000000) as u32;
    //         let value = value << shift;
    //         result |= value;
    //         if byte & 0b10000000 == 0  {
    //             break;
    //         }
    //         shift += 7;
    //     }

    //     // Unsetting the added sign bits
    //     result &= !(0xff_ff_ff_ff << size);
    //     result
    // }

    // fn varint32(&mut self) -> i32 {
    //     const size = 32;
    //     let mut result = 0;
    //     let mut shift = 0;

    //     for byte in bytes {
    //         // Unsetting the msb
    //         let value = (byte & !0b10000000) as u32;
    //         let value = value << shift;
    //         result |= value;
    //         if byte & 0b10000000 == 0  {
    //             break;
    //         }
    //         shift += 7;
    //     }

    //     // Unsetting the added sign bits
    //     result &= !(0xff_ff_ff_ff << size);
    //     result
    // }
}

// pub fn compile(source: Vec<u8>) -> Module {

// }
