use crate::llvm_backend::parser::Parser;
use crate::llvm_backend::parser::Error;

#[cfg(test)]
mod parser_tests {
    use super::{Parser, Error};

    // TODO: Are the following functions better as private?
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
        assert_eq!(result, Error::MalformedVaruint7);
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
        assert_eq!(result, Error::MalformedVaruint32);
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
        assert_eq!(result, Error::MalformedVarint7);
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
        assert_eq!(result, Error::MalformedVarint32);
    }
}
