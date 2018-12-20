use std::collections::HashMap;

/* ------------------------------------------------------------------ */

#[derive(Debug, Clone)]
///
pub struct Module {
    pub signatures: Vec<Vec<u8>>,
    // pub imports: Imports,
    // pub exports: Exports,
    // pub functions: HashMap<String, u32>,
    // pub ir: LLVMModule,
}

///
impl Module {
    pub fn new() -> Self {
        Module {
            signatures: Vec::new(),
        }
    }
}

///
struct Exports {
    pub functions: HashMap<String, u32>,
}


///
struct Imports {
    pub functions: HashMap<(String, String), u32>,
}

/* ------------------------------------------------------------------ */

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    BufferEndReached,
    InvalidVarint7,
    InvalidVaruint7,
    InvalidMagicNumber,
    InvalidVersionNumber,
}

/* ------------------------------------------------------------------ */

#[derive(Debug, Clone)]
/// A single-pass codegen parser.
/// Generates a Module as it deserializes a wasm binary.
pub struct Parser<'a> {
    code: &'a Vec<u8>, // The wasm binary to parse
    cursor: usize, // Used to track the current byte position as the parser advances.
    module: Module, // The generated module
}

/// Contains the implementation of parser
impl <'a> Parser<'a> {
    /// Creates new parser
    pub fn new(code: &'a Vec<u8>) -> Self {
        Parser {
            code,
            cursor: 0, // cursor starts at first byte
            module: Module::new(),
        }
    }

    /// Generates the `module` object by calling functions
    /// that parse a wasm module.
    pub fn module(&mut self) {
        println!("= Parsing wasm module! =");
        // Consume preamble.
        self.preamble().unwrap();
        if let Ok(value) = self.varint7() {
            println!("value = {}, (equals -0x1: {})", value, value == -0x01);
        };
    }

    /// Checks if the following bytes are expected
    /// wasm preamble bytes.
    pub fn preamble(&mut self) -> Result<(), Error> {
        // Consume magic number.
        let magic_no = self.uint32()?;
        // Consume version number.
        let version_no = self.uint32()?;
        println!("magic = 0x{:08x}, version = 0x{:08x}", magic_no, version_no);
        if magic_no != 0x6d736100 {
            return Err(Error::InvalidMagicNumber);
        }
        if version_no != 0x1 {
            return Err(Error::InvalidVersionNumber);
        }
        Ok(())
    }

    #[inline]
    /// Gets a byte from the code buffer and (if available)
    /// advances the cursor.
    fn eat_byte(&mut self) -> Option<u8> {
        let index = self.cursor;
        // Check if range is within code buffer bounds
        if index < self.code.len() {
            // Advance the cursor
            self.cursor += 1;
            return Some(self.code[index]);
        }
        None
    }

    /// Gets the next `range` slice of bytes from the code buffer
    /// (if available) and advances the token.
    fn eat_bytes(&mut self, range: usize) -> Option<&[u8]> {
        let start = self.cursor;
        let end = start + range;
        // Check if range is within code buffer bounds
        if end > self.code.len() {
            return None;
        }
        // Advance the cursor
        self.cursor = end;
        Some(&self.code[start..end])
    }

    /// Consumes 4 bytes that represents 32-bit unsigned integer
    pub fn uint32(&mut self) -> Result<u32, Error> {
        if let Some(bytes) = self.eat_bytes(4) {
            let mut shift = 0;
            let mut result = 0;
            for byte in bytes {
                result |= (*byte as u32) << shift;
                shift += 8;
            }
            return Ok(result);
        }
        Err(Error::BufferEndReached)
    }

    // /// Consumes a byte that represents a 1-bit LEB128 unsigned integer encoding
    // fn varuint1(&mut self) -> Result<u8, Error> {
    //     if let Some(byte) = self.eat_byte() {
    //         let mut result = byte;
    //         // Check if msb is unset.
    //         if result & 0b1000_0000 != 0 { // TODO: Check if test is really needed (fuzz tests and large programs)
    //             return Err(Error::InvalidVaruint7);
    //         }
    //         return Ok(result);
    //     }
    //     Err(Error::BufferEndReached)
    // }

    /// Consumes a byte that represents a 7-bit LEB128 unsigned integer encoding
    fn varuint7(&mut self) -> Result<u8, Error> {
        if let Some(byte) = self.eat_byte() {
            let mut result = byte;
            // Check if msb is unset.
            if result & 0b1000_0000 != 0 { // TODO: Check if test is really needed (fuzz tests and large programs)
                return Err(Error::InvalidVaruint7);
            }
            return Ok(result);
        }
        Err(Error::BufferEndReached)
    }


    // fn varint7(&mut self) -> Option<i8> {
    //     const size: i8 = 7;
    //     let mut result = 0;
    //     let mut shift = 0;

    //     if let Some(byte) = self.eat_byte() {
    //         // for byte in bytes {
    //             // Unset the msb
    //             let value = (byte & !0b10000000) as i8;
    //             let value = value << shift;
    //             result |= value;
    //             shift += 7;
    //         // }

    //         // Unset the added sign bits
    //         result &= !(0xff << size);
    //         return Some(result);
    //     }
    //     None
    // }

    /// Consumes a byte that represents a 7-bit LEB128 signed integer encoding
    fn varint7(&mut self) -> Result<i8, Error> {
        if let Some(byte) = self.eat_byte() {
            let mut result = byte;
            // Check if msb is unset.
            if result & 0b1000_0000 != 0 {
                return Err(Error::InvalidVarint7);
            }
            // If the 7-bit value is signed, extend the sign.
		    if result & 0b0100_0000 == 0b0100_0000 {
                result |= 0b1000_0000;
            }
            println!("value = {:08b}", result);
            return Ok(result as i8);
        }
        Err(Error::BufferEndReached)
    }

    // fn varint7(&mut self) -> Option<i8> {
    //     const size: i8 = 7;
    //     let mut result = 0;
    //     let mut shift = 0;

    //     if let Some(byte) = self.eat_byte() {
    //         // for byte in bytes {
    //             // Unset the msb
    //             let value = (byte & !0b10000000) as i8;
    //             let value = value << shift;
    //             result |= value;
    //             shift += 7;
    //         // }

    //         // Unset the added sign bits
    //         result &= !(0xff << size);
    //         return Some(result);
    //     }
    //     None
    // }

    // fn varint7(&mut self) -> Option<i8> {
    //     const size: i8 = 7;
    //     let mut result = 0;
    //     let mut shift = 0;

    //     if let Some(byte) = self.eat_byte() {
    //         for byte in bytes {
    //             // Unset the msb
    //             let value = (byte & !0b10000000) as u32;
    //             let value = value << shift;
    //             result |= value;
    //             shift += 7;
    //         }

    //         // Unset the added sign bits
    //         result &= !(0xff << size);
    //         return Some(result);
    //     }
    //     None
    // }
}

// pub fn compile(source: Vec<u8>) -> Module {
// }

#[cfg(test)]
mod parser_tests {
    use super::Parser;
    use super::Error;

    #[test]
    fn eat_byte_can_consume_next_byte_if_available() {
        let code = vec![0x6d];
        let mut parser = Parser::new(&code);
        let result = parser.eat_byte().unwrap();
        assert_eq!(result, 0x6d);
    }

    #[test]
    fn eat_byte_can_consume_just_the_next_byte_if_available() {
        let code = vec![0x01, 0x00];
        let mut parser = Parser::new(&code);
        let result = parser.eat_byte().unwrap();
        assert_eq!(result, 0x1);
    }

    #[test]
    fn eat_byte_can_consume_just_the_next_byte_if_available_2() {
        let code = vec![0x01, 0x5f];
        let mut parser = Parser::new(&code);
        // Consume first byte.
        let result = parser.eat_byte();
        // Then consume the next byte.
        let result = parser.eat_byte().unwrap();
        assert_eq!(result, 0x5f);
    }

    #[test]
    fn eat_byte_cannot_consume_next_byte_if_not_available() {
        let code = vec![];
        let mut parser = Parser::new(&code);
        let result = parser.eat_byte();
        assert!(result.is_none());
    }

    #[test]
    fn eat_bytes_can_consume_next_specified_bytes_if_available() {
        let code = vec![0x00, 0x61, 0x73, 0x6d];
        let mut parser = Parser::new(&code);
        let result = parser.eat_bytes(4).unwrap();
        assert_eq!(result, &[0x00, 0x61, 0x73, 0x6d]);
    }

    #[test]
    fn eat_bytes_can_consume_next_specified_bytes_if_available_2() {
        let code = vec![0x00, 0x61, 0x73, 0x6d, 0x1];
        let mut parser = Parser::new(&code);
        let result = parser.eat_bytes(5).unwrap();
        assert_eq!(result, &[0x00, 0x61, 0x73, 0x6d, 0x1]);
    }

    #[test]
    fn eat_bytes_can_consume_next_specified_bytes_if_available_3() {
        let code = vec![0x01, 0x10, 0x73, 0x6d, 0x09, 0xff, 0x5e];
        let mut parser = Parser::new(&code);
        // Consume 4 bytes first.
        let result = parser.eat_bytes(4);
        // Then consume the next 3 bytes.
        let result = parser.eat_bytes(3).unwrap();
        assert_eq!(result, &[0x09, 0xff, 0x5e]);
    }

    #[test]
    fn eat_bytes_can_consume_just_the_next_specified_bytes_if_available() {
        let code = vec![0x01, 0x00, 0x73, 0x00, 0x1];
        let mut parser = Parser::new(&code);
        let result = parser.eat_bytes(1).unwrap();
        assert_eq!(result, &[0x1]);
    }

    #[test]
    fn eat_bytes_cannot_consume_next_specified_bytes_if_not_available() {
        let code = vec![0x01, 0x00, 0x00];
        let mut parser = Parser::new(&code);
        let result = parser.eat_bytes(4);
        assert!(result.is_none());
    }

    #[test]
    fn eat_bytes_cannot_consume_next_specified_bytes_if_not_available_2() {
        let code = vec![0x01, 0x10, 0x73, 0x6d, 0x09, 0xff, 0x5e];
        let mut parser = Parser::new(&code);
        // Consume 5 bytes first.
        let result = parser.eat_bytes(5);
        // Then consume the next 3 bytes.
        let result = parser.eat_bytes(3);
        assert!(result.is_none());
    }

    #[test]
    fn uint32_can_consume_next_4_bytes_if_available() {
        let code = vec![0x00, 0x61, 0x73, 0x6d];
        let mut parser = Parser::new(&code);
        let result = parser.uint32().unwrap();
        assert_eq!(result, 0x6d736100);
    }

    #[test]
    fn uint32_can_consume_just_the_next_4_bytes_if_available() {
        let code = vec![0x01, 0x00, 0x00, 0x00, 0x1];
        let mut parser = Parser::new(&code);
        let result = parser.uint32().unwrap();
        assert_eq!(result, 0x1);
    }

    #[test]
    fn uint32_cannot_consume_next_4_bytes_if_not_available() {
        let code = vec![0x01, 0x00, 0x00];
        let mut parser = Parser::new(&code);
        let result = parser.uint32().unwrap_err();
        assert_eq!(result, Error::BufferEndReached);
    }

    #[test]
    fn uint32_cannot_consume_next_4_bytes_if_not_available_2() {
        let code = vec![];
        let mut parser = Parser::new(&code);
        let result = parser.uint32().unwrap_err();
        assert_eq!(result, Error::BufferEndReached);
    }

    #[test]
    fn varint7_can_consume_next_byte_if_available_and_valid() {
        let code = vec![0x7f, 0x00, 0x00];
        let mut parser = Parser::new(&code);
        let result = parser.varint7().unwrap();
        assert_eq!(result, -0x1);
    }

    #[test]
    fn varint7_can_consume_next_byte_if_available_and_valid_2() {
        let code = vec![0x60];
        let mut parser = Parser::new(&code);
        let result = parser.varint7().unwrap();
        assert_eq!(result, -0x20);
    }

    #[test]
    fn varint7_cannot_consume_next_byte_if_not_available() {
        let code = vec![];
        let mut parser = Parser::new(&code);
        let result = parser.varint7().unwrap_err();
        assert_eq!(result, Error::BufferEndReached);
    }

    #[test]
    fn varint7_cannot_consume_next_byte_if_not_valid_varint7() {
        let code = vec![0b1000_0000];
        let mut parser = Parser::new(&code);
        let result = parser.varint7().unwrap_err();
        assert_eq!(result, Error::InvalidVarint7);
    }

    #[test]
    fn varuint7_can_consume_next_byte_if_available_and_valid_2() {
        let code = vec![0b0100_0000];
        let mut parser = Parser::new(&code);
        let result = parser.varuint7().unwrap();
        assert_eq!(result, 0b0100_0000);
    }

    #[test]
    fn varuint7_cannot_consume_next_byte_if_not_available() {
        let code = vec![];
        let mut parser = Parser::new(&code);
        let result = parser.varuint7().unwrap_err();
        assert_eq!(result, Error::BufferEndReached);
    }

    #[test]
    fn varuint7_cannot_consume_next_byte_if_not_valid_varuint7() {
        let code = vec![0b1000_0000];
        let mut parser = Parser::new(&code);
        let result = parser.varuint7().unwrap_err();
        assert_eq!(result, Error::InvalidVaruint7);
    }
}

