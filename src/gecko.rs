use thiserror::Error;
use std::io::Cursor;

use crate::ppc;

// This is NOT a conclusive list of every type of gecko code.
// Instead, it consists of commonly-used types.
// Feel free to request that a code type be implemented.
// pub enum GeckoCodeType {
//     // U8RAMWrite {
//     //     address: u32,
//     //     count: u16,
//     //     value: u8,
//     // },

//     // U16RAMWrite {
//     //     address: u32,
//     //     count: u16,
//     //     value: u16
//     // },

//     // U32RAMWrite {
//     //     address: u32,
//     //     value: u32
//     // },

//     // StringWrite {
//     //     address: u32,
//     //     value: String
//     // },

//     /// A branch to a subroutine containing `code` will
//     /// be placed at `address`. The code must end with
//     /// `0x00000000`. If an additional line must be used
//     /// to do this, use a `nop`, (`0x60000000`).
//     InsertAssembly {
//         address: u32,
//         code: Vec<u32>
//     }
// }

/* Util */
#[derive(Error, Debug)]
pub enum GeckoCodeConversionError {
    // #[error("Unimplemented")]
    // Unimplemented,

    #[error("Invalid gecko code type. Line number: {line_number}, found value: 0x{:08X}", value)]
    InvalidType {
        line_number: usize,
        value: u32
    },

    #[error("Malformed gecko code")]
    Malformed,

    #[error("Empty gecko code")]
    Empty
}

fn get_and_seek(cursor: &mut Cursor<&[u32]>) -> u32 {
    let pos = cursor.position();
    let value = cursor.get_ref()[pos as usize];
    cursor.set_position(pos + 1);
    value
}

fn get_code_address(cursor: &mut Cursor<&[u32]>, larger_address: bool) -> u32 {
    let address = get_and_seek(cursor) & 0x00FFFFFF;

    let final_address = 0x80000000 | address;
    
    if larger_address {
        final_address + 0x01000000
    } else {
        final_address
    }
}



pub fn convert_from_gecko_code_values(gecko_code: &[u32]) -> Result<String, GeckoCodeConversionError> {
    let code_length = gecko_code.len();

    // make sure the code is valid length-wise

    if code_length == 0 {
        return Err(GeckoCodeConversionError::Empty);
    } else if code_length % 2 != 0 {
        return Err(GeckoCodeConversionError::Malformed);
    }

    let mut cursor = Cursor::new(gecko_code);

    let mut result = String::new();

    let mut current_cursor_position = 0;
    while current_cursor_position < gecko_code.len() {
        let current_value = gecko_code[current_cursor_position];

        // detect code type -- this is the first byte in the code sequence
        let byte = ((current_value & 0xFF000000) >> 0x18) as u8;

        match byte {
            // // 8-bit RAM Write
            // 0x00 | 0x01 => {

            // }

            // 16-bit RAM Write & Fill
            0x02 | 0x03 => {
                result += &from_02(&mut cursor, byte % 2 != 0)?;
            }
            
            // 32-bit RAM Write
            0x04 | 0x05 => {
                result += &from_04(&mut cursor, byte % 2 != 0)?;
            }

            // String RAM Write
            0x06 => {
                result += &from_06(&mut cursor, byte % 2 != 0)?;
            }
            
            // Set Gecko Register to
            0x80 => {
                result += &from_80(&mut cursor)?;
            }

            // Load into Gecko Register
            0x82 =>  {
                result += &from_82(&mut cursor)?;
            }

            // Insert Assembly
            0xC2 | 0xC3 => {
                result += &from_c2(&mut cursor, byte % 2 != 0)?;
            }

            // Create a Branch
            0xC6 | 0xC7 => {
                result += &from_c6(&mut cursor, byte % 2 != 0)?;
            }

            // Invalid/Unsupported
            _ => {
                let err = GeckoCodeConversionError::InvalidType {
                    line_number: (current_cursor_position / 2) + 1,
                    value: current_value
                };
                
                return Err(err);
            }
        }

        result += "\n// ---\n";
        current_cursor_position = cursor.position() as usize;
    }

    Ok(result)
}


/* Code Types */

/// # 0x00: 8-bit RAM Write & Fill
/// The `value` will **constantly** fill the range `address`
/// to `address + count + 1`.
// fn from_00(cursor: &mut Cursor<&[u32]>, larger_address: bool) -> Result<String, GeckoCodeConversionError> {
//     // let mut result = "// Constant 8-bit RAM "
//     Ok(String::new())
// }

/// # 0x02: 16-bit RAM Write & Fill
/// The `value` will **constantly** fill the range
/// `address` to `address + count + 1`.
/// ## Parameters
/// `cursor`: The `Cursor` for the gecko code.
/// `larger_address`: Indicates if the given address is >= `0x01000000`.
/// ## Returns
/// `Result<String, GeckoCodeConversionError>`
fn from_02(cursor: &mut Cursor<&[u32]>, larger_address: bool) -> Result<String, GeckoCodeConversionError> {
    let mut result = "// - Constant 16-bit RAM Fill -\n".to_string();
    let address = get_code_address(cursor, larger_address);
    let temp = get_and_seek(cursor);

    let count = (temp & 0xFFFF0000) >> 0x10;
    let value = (temp & 0x0000FFFF) as u16;
    result += &format!("// Range: 0x{:08X} to 0x{:08X}\n", address, address + count + 1);
    result += &format!("// Value: 0x{:08X}", value);
    
    Ok(result)
}

/// # 0x04: 32-bit RAM Write
/// The specified `value` will **constantly** be
/// written to `address`.
/// ## Parameters
/// `cursor`: The `Cursor` for the gecko code.
/// `larger_address`: Indicates if the given address is >= `0x01000000`.
/// ## Returns
/// `Result<String, GeckoCodeConversionError>`
fn from_04(cursor: &mut Cursor<&[u32]>, larger_address: bool) -> Result<String, GeckoCodeConversionError> {
    let mut result = "// - Constant 32-bit RAM Write -\n".to_string();
    result += &format!("// Target address: 0x{:08X}\n", get_code_address(cursor, larger_address));
    result += &format!("// Value: 0x{:08X}", get_and_seek(cursor));
    Ok(result)
}

/// # 0x06: String RAM Write
/// The following `count` bytes will be written to `address`.
/// ### Note
/// The name of the code type is "String" RAM Write, but
/// there is no null-termination check to ensure that the
/// contents are actually a valid string. In other words,
/// this code type can simply be used to write raw bytes,
/// regardless of content.
/// ## Parameters
/// `cursor`: The `Cursor` for the gecko code.
/// `larger_address`: Indicates if the given address is >= `0x01000000`.
/// ## Returns
/// `Result<String, GeckoCodeConversionError>`
fn from_06(cursor: &mut Cursor<&[u32]>, larger_address: bool) -> Result<String, GeckoCodeConversionError> {
    let mut result = "// - String RAM Write - \n".to_string();
    result += &format!("// Target address: 0x{:08X}\n", get_code_address(cursor, larger_address));
    let num_bytes = get_and_seek(cursor);

    // determine the number of values to skip
    let num_values = (num_bytes as usize).next_multiple_of(4) / 4;

    // read raw bytes
    let mut raw_bytes: Vec<u8> = Vec::new();

    for _ in 0..num_values {
        let value = get_and_seek(cursor);

        // the bytes must be in big endian before adding
        // them to the list

        let bytes = value.to_be_bytes();
        raw_bytes.extend(bytes);
    }

    // discard extraneous values
    raw_bytes.resize(num_bytes as usize, 0);

    // determine if the bytes can be output as a string
    // or if they should be output as-is
    let mut is_string = false;

    if let Some(index) = raw_bytes
        .iter()
        .position(|byte| *byte == 0)
    {
        if !(index < raw_bytes.len() - 1) {
            // the only 0 is at the end; this can
            // be considered a *candidate* for 
            // a valid string
            is_string = true;
        }
    }

    // determine if the string was valid and printable
    let mut printed_string = false;

    if is_string {
        // try to convert it to a string
        if let Ok(string) = String::from_utf8(raw_bytes.to_vec()) {
            printed_string = true;
            result += &format!("// String contents: \"{string}\"\n");
        }
    }

    if !is_string || !printed_string {
        // not a string or the string wasn't printable
        // print out bytes instead
        
        result += "// Byte contents: [";

        
        for (index, byte) in raw_bytes.iter().enumerate() {

            // check if this is the last one
            if index == raw_bytes.len() - 1 {
                result += &format!("0x{:02X}]", byte);
            } else {
                result += &format!("0x{:02X}, ", byte);
            }
        }
    }
    
    
    Ok(result)
}

/// # 0x80: Set Gecko Register to
/// ## Parameters
/// `cursor`: The `Cursor` for the gecko code.
/// ## Returns
/// `Result<String, GeckoCodeConversionError>`
fn from_80(cursor: &mut Cursor<&[u32]>) -> Result<String, GeckoCodeConversionError> {
    let register = get_and_seek(cursor) & 0x000000FF;
    let value = get_and_seek(cursor);

    Ok(format!("// - Set Gecko Register {register} to 0x{:08X} -", value))
}

/// # 0x82: Load into Gecko Register
/// ## Parameters
/// `cursor`: The `Cursor` for the gecko code.
/// ## Returns
/// `Result<String, GeckoCodeConversionError>`
fn from_82(cursor: &mut Cursor<&[u32]>) -> Result<String, GeckoCodeConversionError> {
    let register = get_and_seek(cursor) & 0x000000FF;
    let value = get_and_seek(cursor);

    Ok(format!("// - Load value 0x{:08X} into register {register}", value))
}

/// # 0xC2: Insert Assembly
/// A branch to a subroutine containing `code` will
/// be placed at `address`. The code must end with
/// `0x00000000`. If an additional line must be used
/// to do this, use a `nop` (`0x60000000`). The value
/// stored in the second value of the first line is
/// the total number of *subsequent* lines. **The Gecko
/// Code handler will automatically add a branch back to
/// `address + 0x4`.**
/// ## Parameters
/// `cursor`: The `Cursor` for the gecko code.
/// `larger_address`: Indicates if the given address is >= `0x01000000`.
/// ## Returns
/// `Result<String, GeckoCodeConversionError>`
fn from_c2(cursor: &mut Cursor<&[u32]>, larger_address: bool) -> Result<String, GeckoCodeConversionError> {
    let mut result = "// - Insert Assembly -\n".to_string();

    // find address
    let address = get_code_address(cursor, larger_address);
    result += &format!("// Target address: 0x{:08X}\n\n", address);

    let _num_lines = get_and_seek(cursor);

    let cursor_len = cursor.get_ref().len();

    // process assembly
    while (cursor.position() as usize) < cursor_len {
        let left_code = get_and_seek(cursor);
        let right_code = get_and_seek(cursor);

        // check if this is the end of the code
        if left_code == 0x60000000 {
            break;
        }

        result += &(ppc::code_to_instruction(left_code) + "\n");

        // check if this is the end of the code
        if right_code == 0 || right_code == 0x60000000 {
            break;
        }

        result += &(ppc::code_to_instruction(right_code) + "\n");
    }

    Ok(result)
}

/// # 0xC6: Create a Branch
/// A branch to `target` is placed at `address`.
/// ## Parameters
/// `cursor`: The `Cursor` for the gecko code.
/// `larger_address`: Indicates if the given address is >= `0x01000000`.
/// ## Returns
/// `Result<String, GeckoCodeConversionError>`
fn from_c6(cursor: &mut Cursor<&[u32]>, larger_address: bool) -> Result<String, GeckoCodeConversionError> {
    let mut result = "// - Create a Branch -\n".to_string();
    result += &format!("// Target address: 0x{:08X}\n", get_code_address(cursor, larger_address));
    result += &format!("// Branch to: 0x{:08X}\n", get_and_seek(cursor));
    Ok(result)
}