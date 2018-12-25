use std::collections::HashMap;
// use llvm_sys;

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
    // Storage
    InvalidVaruint1,
    InvalidVaruint7,
    InvalidVarint7,
    InvalidVaruint32,
    InvalidVarint32,
    InvalidVarint64,
    // Types
    InvalidValueType,
    // ExternalKind
    InvalidImportType,
    // Preamble
    InvalidMagicNumber,
    InvalidVersionNumber,
    // Sections
    SectionAlreadyDefined,
    UnsupportedSection,
    InvalidSectionId,
    SectionPayloadDoesNotMatchPayloadLength,
    // Custom Section
    IncompleteCustomSection,
    InvalidPayloadLengthInCustomSection,
    InvalidNameLengthInCustomSection,
    // Type Section
    IncompleteTypeSection,
    InvalidPayloadLengthInTypeSection,
    InvalidEntryCountInTypeSection,
    EntriesDoNotMatchEntryCountInTypeSection,
    InvalidTypeInTypeSection,
    UnsupportedTypeInTypeSection,
    // Import Entry
    IncompleteImportEntry,
    InvalidModuleLengthInImportEntry,
    ModuleStringDoesNotMatchModuleLengthInImportEntry,
    InvalidFieldLengthInImportEntry,
    FieldStringDoesNotMatchFieldLengthInImportEntry,
    InvalidImportTypeInImportEntry,
    // Function Type
    IncompleteFunctionType,
    InvalidParamCountInFunctionType,
    ParamsDoesNotMatchParamCountInFunctionType,
    InvalidParamTypeInFunctionType,
    InvalidReturnCountInFunctionType,
    InvalidReturnTypeInFunctionType,
    ReturnTypeDoesNotMatchReturnCountInFunctionType,
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

    /// TODO: TEST
    /// Generates the `module` object by calling functions
    /// that parse a wasm module.
    pub fn module(&mut self) {
        println!("\n=== module! ===");

        // Consume preamble. Panic if it returns an error.
        self.module_preamble().unwrap();
        // Error::BufferEndReached => MalformedWasmModule
        // Error::InvalidMagicNumber => same
        // Error::InvalidVersionNumber => same

        self.module_sections().unwrap(); // Optional
        // Error::BufferEndReached => MalformedWasmModule
        // ...
    }

    /// TODO: TEST
    /// Checks if the following bytes are expected
    /// wasm preamble bytes.
    pub fn module_preamble(&mut self) -> Result<(), Error> {
        println!("\n=== module_preamble! ===");

        // Consume magic number.
        let magic_no = self.uint32()?;

        // Consume version number.
        let version_no = self.uint32()?;

        println!("\n::module_preamble::magic_no = 0x{:08x}", magic_no);

        println!("\n::module_preamble::version_no = 0x{:08x}", version_no);

        // Magic number must be `\0asm`
        if magic_no != 0x6d736100 {
            return Err(Error::InvalidMagicNumber);
        }

        // Only version 0x01 supported for now.
        if version_no != 0x1 {
            return Err(Error::InvalidVersionNumber);
        }

        Ok(())
    }

    /// TODO: TEST
    pub fn module_sections(&mut self) -> Result<(), (Error, usize)> {
        println!("\n=== module_sections! ===");

        //
        let mut sections_consumed = vec![];

        //
        loop {
            let start_position = self.cursor;
            //
            let section_id = match self.varuint7() {
                Ok(value) => value,
                Err(error) => {
                    //
                    if error == Error::BufferEndReached {
                        break;
                    } else {
                        return Err((Error::InvalidSectionId, start_position));
                    }
                },
            };

            //
            if sections_consumed.contains(&section_id) {
                return Err((Error::SectionAlreadyDefined, start_position));
            } else {
                sections_consumed.push(section_id);
            }

            //
            match section_id {
                0x00 => self.custom_section()?,
                0x01 => self.type_section()?,
                0x02 => self.import_section()?,
                _ => {
                    return Err((Error::UnsupportedSection, start_position));
                },
            };
        }
        Ok(())
    }

    /// TODO: TEST
    pub fn custom_section(&mut self) -> Result<(), (Error, usize)> {
        println!("\n=== custom_section! ===");
        let start_position = self.cursor;

        //
        let payload_len = match self.varint32() {
            Ok(value) => value,
            Err(error) => {
                //
                if error == Error::BufferEndReached {
                    return Err((Error::IncompleteCustomSection, start_position));
                } else {
                    return Err((Error::InvalidPayloadLengthInCustomSection, start_position));
                }
            }
        };

        //
        let name_len = match self.varint32() {
            Ok(value) => value,
            Err(error) => {
                //
                if error == Error::BufferEndReached {
                    return Err((Error::IncompleteCustomSection, start_position));
                } else {
                    return Err((Error::InvalidEntryCountInTypeSection, start_position));
                }
            }
        };

        {
            // Skip payload bytes
            let _name = match self.eat_bytes(name_len as _) {
                Some(value) => value,
                None => {
                    return Err((Error::IncompleteCustomSection, start_position));
                }
            };
        }

        // Skip payload bytes
        let _payload_data = match self.eat_bytes(payload_len as _) {
            Some(value) => value,
            None => {
                return Err((Error::IncompleteCustomSection, start_position));
            }
        };

        Ok(())
    }

    /// TODO: TEST
    pub fn type_section(&mut self) -> Result<(), (Error, usize)> {
        println!("\n=== type_section! ===");
        let start_position = self.cursor;

        //
        let payload_len = match self.varuint32() {
            Ok(value) => value,
            Err(error) => {
                //
                if error == Error::BufferEndReached {
                    return Err((Error::IncompleteTypeSection, start_position));
                } else {
                    return Err((Error::InvalidPayloadLengthInTypeSection, start_position));
                }
            }
        };

        println!("\n::type_section::payload_len = 0x{:x}", payload_len);

        //
        let entry_count = match self.varuint32() {
            Ok(value) => value,
            Err(error) => {
                //
                if error == Error::BufferEndReached {
                    return Err((Error::IncompleteTypeSection, start_position));
                } else {
                    return Err((Error::InvalidEntryCountInTypeSection, start_position));
                }
            }
        };

        println!("\n::type_section::entry_count = 0x{:x}", entry_count);

        //
        for i in 0..entry_count {
            let type_id = match self.varint7() {
                Ok(value) => value,
                Err(error) => {
                    //
                    if error == Error::BufferEndReached {
                        return Err((Error::EntriesDoNotMatchEntryCountInTypeSection, start_position));
                    } else {
                        return Err((Error::InvalidTypeInTypeSection, start_position));
                    }
                },
            };

            println!("\n::type_section::type_id = {:?}", type_id);

            match type_id {
                -0x20 => self.func_type()?,
                _ => {
                    return Err((Error::UnsupportedTypeInTypeSection, start_position));
                },
            };
        }

        Ok(())
    }

    /// TODO: TEST
    pub fn import_section(&mut self) -> Result<(), (Error, usize)> {
        println!("\n=== import_section! ===");
        let start_position = self.cursor;

        //
        let payload_len = match self.varuint32() {
            Ok(value) => value,
            Err(error) => {
                //
                if error == Error::BufferEndReached {
                    return Err((Error::IncompleteTypeSection, start_position));
                } else {
                    return Err((Error::InvalidPayloadLengthInTypeSection, start_position));
                }
            }
        };
        println!("\n::import_section::payload_len = 0x{:x}", payload_len);

        //
        let entry_count = match self.varuint32() {
            Ok(value) => value,
            Err(error) => {
                //
                if error == Error::BufferEndReached {
                    return Err((Error::IncompleteTypeSection, start_position));
                } else {
                    return Err((Error::InvalidEntryCountInTypeSection, start_position));
                }
            }
        };

        println!("\n::import_section::entry_count = 0x{:x}", entry_count);

        //
        for i in 0..entry_count {
            // Throwing away type for now!
            self.import_entry()?;
        }

        Ok(())
    }

    /// TODO: TEST
    // pub fn import_entry(&mut self) -> Result<(), (Error, usize)> {
    //     println!("\n=== import_entry! ===");
    //     let start_position = self.cursor;

    //     //
    //     let module_len = match self.varuint32() {
    //         Ok(value) => value,
    //         Err(error) => {
    //             //
    //             if error == Error::BufferEndReached {
    //                 return Err((Error::IncompleteImportEntry, start_position));
    //             } else {
    //                 return Err((Error::InvalidModuleLengthInImportEntry, start_position));
    //             }
    //         }
    //     };

    //     println!("\n::import_entry::module_len = 0x{:x}", payload_len);

    //     let _module_str = match self.eat_bytes(name_len as _) {
    //         Some(value) => value,
    //         None => {
    //             return Err((Error::IncompleteImportEntry, start_position));
    //         }
    //     };

    //     println!("\n::import_entry::_module_str = {:?}", std::str::from_utf8(_module_str));

    //     //
    //     let field_len = match self.varint32() {
    //         Ok(value) => value,
    //         Err(error) => {
    //             //
    //             if error == Error::BufferEndReached {
    //                 return Err((Error::IncompleteImportEntry, start_position));
    //             } else {
    //                 return Err((Error::InvalidFieldLengthInImportEntry, start_position));
    //             }
    //         }
    //     };

    //     println!("\n::import_entry::field_len = 0x{:x}", payload_len);

    //     let _field_str = match self.eat_bytes(name_len as _) {
    //         Some(value) => value,
    //         None => {
    //             return Err((Error::IncompleteImportEntry, start_position));
    //         }
    //     };

    //     println!("\n::import_entry::_field_str = {:?}", std::str::from_utf8(_module_str));

    //     let external_kind =  match self.external_kind() {
    //         Ok(value) => value,
    //         Err(error) => {
    //             if error == Error::BufferEndReached {
    //                 return Err((Error::IncompleteImportEntry, start_position));
    //             } else {
    //                 return Err((Error::InvalidImportTypeInImportEntry, start_position));
    //             }
    //         }
    //     };

    //     match external_kind {
    //         // Function import
    //         0x01 => self.function_import(),
    //         // Table import
    //         0x02 => self.table_import(),
    //         // Memory import
    //         0x03 => self.memory_import(),
    //         // Global import
    //         0x04 => self.global_import(),
    //     }

    //     Ok(())
    // }

    //
    pub fn func_type(&mut self) -> Result<(), (Error, usize)> {
        println!("\n=== func_type! ===");
        let start_position = self.cursor;

        //
        let param_count = match self.varint32() {
            Ok(value) => value,
            Err(error) => {
                //
                if error == Error::BufferEndReached {
                    return Err((Error::IncompleteFunctionType, start_position));
                } else {
                    return Err((Error::InvalidParamCountInFunctionType, start_position));
                }
            }
        };

        println!("\n::func_type::param_count = 0x{:x}", param_count);

        //
        for i in 0..param_count {
            // Throwing away type for now!
            let param_type = match self.value_type() {
                Ok(value) => value,
                Err(error) => {
                    if error == Error::BufferEndReached {
                        return Err((Error::IncompleteFunctionType, start_position));
                    } else {
                        return Err((Error::InvalidParamTypeInFunctionType, start_position));
                    }
                }
            };

            println!("\n::func_type::param_type = {:?}", param_type);
        }

        //
        let return_count = match self.varuint1() {
            Ok(value) => value,
            Err(error) => {
                //
                if error == Error::BufferEndReached {
                    return Err((Error::IncompleteFunctionType, start_position));
                } else {
                    return Err((Error::InvalidReturnCountInFunctionType, start_position));
                }
            }
        };

        println!("\n::func_type::return_count = {:?}", return_count);

        if return_count {
            // Throwing away type for now!
            let return_type = match self.value_type() {
                Ok(value) => value,
                Err(error) => {
                    if error == Error::BufferEndReached {
                        return Err((Error::IncompleteFunctionType, start_position));
                    } else {
                        return Err((Error::InvalidReturnTypeInFunctionType, start_position));
                    }
                }
            };

            println!("\n::func_type::return_type = {:?}", return_type);
        }

        Ok(())
    }

    #[inline]
    //
    pub fn value_type(&mut self) -> Result<i8, Error> {
        println!("\n=== value_type! ===");

        let value = self.varint7()?;

        // i32, i64, f32, f64
        if value == -0x01 || value == -0x02 || value == -0x03 || value == -0x04 {
            Ok(value as _)
        } else {
            Err(Error::InvalidValueType)
        }
    }

    #[inline]
    //
    pub fn external_kind(&mut self) -> Result<u8, Error> {
        println!("\n=== external_kind! ===");

        let value = self.uint8()?;

        // function_import, table_import, memory_imoort, global_import
        if value == 0x01 || value == 0x02 || value == 0x03 || value == 0x04 {
            Ok(value as _)
        } else {
            Err(Error::InvalidImportType)
        }
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

    /// Consumes 1 byte that represents a 8-bit unsigned integer
    fn uint8(&mut self) -> Result<u8, Error> {
        if let Some(byte) = self.eat_byte() {
            return Ok(byte);
        }
        Err(Error::BufferEndReached)
    }

    /// Consumes 2 bytes that represent a 16-bit unsigned integer
    fn uint16(&mut self) -> Result<u16, Error> {
        if let Some(bytes) = self.eat_bytes(2) {
            let mut shift = 0;
            let mut result = 0;
            for byte in bytes {
                result |= (*byte as u16) << shift;
                shift += 8;
            }
            return Ok(result);
        }
        Err(Error::BufferEndReached)
    }

    /// Consumes 4 bytes that represent a 32-bit unsigned integer
    fn uint32(&mut self) -> Result<u32, Error> {
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

    /// Consumes a byte that represents a 1-bit LEB128 unsigned integer encoding
    fn varuint1(&mut self) -> Result<bool, Error> {
        if let Some(byte) = self.eat_byte() {
            return match byte {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(Error::InvalidVaruint1),
            };
        }
        // We expect the if statement to return an Ok result. If it doesn't
        // then we are trying to read more than 1 byte, which is invalid for a varuint1
        Err(Error::BufferEndReached)
    }

    /// Consumes a byte that represents a 7-bit LEB128 unsigned integer encoding
    fn varuint7(&mut self) -> Result<u8, Error> {
        if let Some(byte) = self.eat_byte() {
            let mut result = byte;
            // Check if msb is unset.
            if result & 0b1000_0000 != 0 {
                return Err(Error::InvalidVaruint7);
            }
            return Ok(result);
        }
        // We expect the if statement to return an Ok result. If it doesn't
        // then we are trying to read more than 1 byte, which is invalid for a varuint7
        Err(Error::BufferEndReached)
    }

    /// Consumes 1-5 bytes that represent a 32-bit LEB128 unsigned integer encoding
    fn varuint32(&mut self) -> Result<u32, Error> {
        // println!("= varuint32! ===");
        let mut result = 0;
        let mut shift = 0;
        while shift < 35 {
            let byte = match self.eat_byte() {
                Some(value) => value,
                None => return Err(Error::BufferEndReached),
            };
            // println!("count = {}, byte = 0b{:08b}", count, byte);
            // Unset the msb and shift by multiples of 7 to the left
            let value = ((byte & !0b10000000) as u32) << shift;
            result |= value;
            // Return if any of the bytes has an unset msb
            if byte & 0b1000_0000 == 0 {
                return Ok(result);
            }
            shift += 7;
        }
        // We expect the loop to terminate early and return an Ok result. If it doesn't
        // then we are trying to read more than 5 bytes, which is invalid for a varuint32
        Err(Error::InvalidVaruint32)
    }

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
            return Ok(result as i8);
        }
        // We expect the if statement to return an Ok result. If it doesn't
        // then we are trying to read more than 1 byte, which is invalid for a varint7
        Err(Error::BufferEndReached)
    }

    /// Consumes 1-5 bytes that represent a 32-bit LEB128 signed integer encoding
    fn varint32(&mut self) -> Result<i32, Error> {
        // println!("= varint32! ===");
        let mut result = 0;
        let mut shift = 0;
        // Can consume at most 5 bytes
        while shift < 35 { // (shift = 0, 7, 14 .. 35)
            let byte = match self.eat_byte() {
                Some(value) => value,
                None => return Err(Error::BufferEndReached),
            };
            // println!("count = {}, byte = 0b{:08b}", count, byte);
            // Unset the msb and shift by multiples of 7 to the left
            let value = ((byte & !0b10000000) as i32) << shift;
            result |= value;
            // Return if any of the bytes has an unset msb
            if byte & 0b1000_0000 == 0 {
                // Extend sign if sign bit is set. We don't bother when we are on the 5th byte
                // (hence shift < 28) because it gives an 32-bit value, so no need for sign
                // extension there
                if shift < 28 && byte & 0b0100_0000 != 0 {
                    result |= -1 << (7 + shift); // -1 == 0xff_ff_ff_ff
                }
                return Ok(result);
            }
            shift += 7;
        }
        // We expect the loop to terminate early and return an Ok result. If it doesn't
        // then we are trying to read more than 5 bytes, which is invalid for a varint32
        Err(Error::InvalidVarint32)
    }

    /// TODO: TEST
    /// Consumes 1-9 bytes that represent a 64-bit LEB128 signed integer encoding
    fn varint64(&mut self) -> Result<i64, Error> {
        // println!("= varint64! ===");
        let mut result = 0;
        let mut shift = 0;
        // Can consume at most 9 bytes
        while shift < 63 { // (shift = 0, 7, 14 .. 56)
            let byte = match self.eat_byte() {
                Some(value) => value,
                None => return Err(Error::BufferEndReached),
            };
            // println!("count = {}, byte = 0b{:08b}", count, byte);
            // Unset the msb and shift by multiples of 7 to the left
            let value = ((byte & !0b10000000) as i64) << shift;
            result |= value;
            // Return if any of the bytes has an unset msb
            if byte & 0b1000_0000 == 0 {
                // Extend sign if sign bit is set. We don't bother when we are on the 9th byte
                // (hence shift < 56) because it gives an 64-bit value, so no need for sign
                // extension there
                if shift < 56 && byte & 0b0100_0000 != 0 {
                    result |= -1 << (7 + shift); // -1 == 0xff_ff_ff_ff
                }
                return Ok(result);
            }
            shift += 7;
        }
        // We expect the loop to terminate early and return an Ok result. If it doesn't
        // then we are trying to read more than 5 bytes, which is invalid for a varint32
        Err(Error::InvalidVarint64)
    }
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
    fn uint8_can_consume_next_byte_if_available() {
        let code = vec![0x22];
        let mut parser = Parser::new(&code);
        let result = parser.uint8().unwrap();
        assert_eq!(result, 0x22);
    }

    #[test]
    fn uint8_can_consume_just_the_next_byte_if_available() {
        let code = vec![0x00, 0x61, 0x73, 0x6d];
        let mut parser = Parser::new(&code);
        let result = parser.uint8().unwrap();
        assert_eq!(result, 0x00);
    }

    #[test]
    fn uint8_cannot_consume_next_byte_if_not_available_2() {
        let code = vec![];
        let mut parser = Parser::new(&code);
        let result = parser.uint8().unwrap_err();
        assert_eq!(result, Error::BufferEndReached);
    }

    #[test]
    fn uint16_can_consume_next_2_bytes_if_available() {
        let code = vec![0x00, 0x61];
        let mut parser = Parser::new(&code);
        let result = parser.uint16().unwrap();
        assert_eq!(result, 0x6100);
    }

    #[test]
    fn uint16_can_consume_just_the_next_2_bytes_if_available() {
        let code = vec![0x01, 0x00, 0x73, 0x6d];
        let mut parser = Parser::new(&code);
        let result = parser.uint16().unwrap();
        assert_eq!(result, 0x1);
    }

    #[test]
    fn uint16_cannot_consume_next_2_bytes_if_not_available() {
        let code = vec![0x01];
        let mut parser = Parser::new(&code);
        let result = parser.uint16().unwrap_err();
        assert_eq!(result, Error::BufferEndReached);
    }

    #[test]
    fn uint16_cannot_consume_next_2_bytes_if_not_available_2() {
        let code = vec![];
        let mut parser = Parser::new(&code);
        let result = parser.uint16().unwrap_err();
        assert_eq!(result, Error::BufferEndReached);
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
    fn varuint7_can_consume_next_byte_if_available_and_valid() {
        let code = vec![0b0111_0100];
        let mut parser = Parser::new(&code);
        let result = parser.varuint7().unwrap();
        assert_eq!(result, 0b0111_0100);
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

    #[test]
    fn varuint32_can_consume_next_bytes_if_available_and_valid() {
        let code = vec![0b1000_0000, 0b1000_0000, 0b1000_0000, 0b1000_0000, 0b0000_1000];
        let mut parser = Parser::new(&code);
        let result = parser.varuint32().unwrap();
        assert_eq!(result, 0b1000_0000_0000_0000_0000_0000_0000_0000);
    }

    #[test]
    fn varuint32_can_consume_next_bytes_if_available_and_valid_2() {
        let code = vec![0b1111_1111, 0b1111_1111, 0b0000_0011, 0b1010_1010];
        let mut parser = Parser::new(&code);
        let result = parser.varuint32().unwrap();
        assert_eq!(result, 0b0000_0000_0000_0000_1111_1111_1111_1111);
    }

    #[test]
    fn varuint32_cannot_consume_next_bytes_if_not_available() {
        let code = vec![0b1000_0000];
        let mut parser = Parser::new(&code);
        let result = parser.varuint32().unwrap_err();
        assert_eq!(result, Error::BufferEndReached);
    }

    #[test]
    fn varuint32_cannot_consume_next_bytes_if_not_valid_varuint32() {
        let code = vec![0b1000_0000, 0b1000_0000, 0b1000_0000, 0b1000_0000, 0b1000_1000];
        let mut parser = Parser::new(&code);
        let result = parser.varuint32().unwrap_err();
        assert_eq!(result, Error::InvalidVaruint32);
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
    fn varint32_can_consume_next_bytes_if_available_and_valid() {
        let code = vec![0b1000_0000, 0b1000_0000, 0b1000_0000, 0b1000_0000, 0b0111_1000,];
        let mut parser = Parser::new(&code);
        let result = parser.varint32().unwrap();
        assert_eq!(result, -2147483648);
    }

    #[test]
    fn varint32_can_consume_next_bytes_if_available_and_valid_2() {
        let code = vec![0b1110_0000, 0b1010_1011, 0b1110_1101, 0b0111_1101, 0b0011_0110];
        let mut parser = Parser::new(&code);
        let result = parser.varint32().unwrap();
        assert_eq!(result, -4_500_000);
    }

    #[test]
    fn varint32_cannot_consume_next_bytes_if_not_available() {
        let code = vec![0b1000_0000, 0b1010_1011, 0b1110_1101];
        let mut parser = Parser::new(&code);
        let result = parser.varint32().unwrap_err();
        assert_eq!(result, Error::BufferEndReached);
    }

    #[test]
    fn varint32_cannot_consume_next_bytes_if_not_valid_varint32() {
        let code = vec![0b1000_0000, 0b1000_0000, 0b1000_0000, 0b1000_0000, 0b1000_1000];
        let mut parser = Parser::new(&code);
        let result = parser.varint32().unwrap_err();
        assert_eq!(result, Error::InvalidVarint32);
    }
}

