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

#[derive(Error, Debug)]
pub enum GeckoCodeConversionError {
    #[error("Invalid gecko code type")]
    InvalidType,

    #[error("Malformed gecko code")]
    Malformed,

    #[error("Empty gecko code")]
    Empty
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
        
        // detect code type -- this is the first byte in the code sequence
        let byte = ((gecko_code[current_cursor_position] & 0xFF000000) >> 0x18) as u8;

        match byte {
            // 32-bit RAM Write
            0x04 | 0x5 => {
                result += &(from_04(&mut cursor, byte % 2 != 0)? + "\n");
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
                result += &(from_c2(&mut cursor, byte % 2 != 0)? + "\n");
            }

            // Invalid/Unsupported
            _ => {
                return Err(GeckoCodeConversionError::InvalidType)
            }
        }

        current_cursor_position = cursor.position() as usize;
    }

    Ok(result)
}


/* Code Types */

/// # 0x04: 32-bit RAM write
/// The specified `value` will **constantly** be
/// written to `address`.
/// ## Parameters
/// `cursor`: The `Cursor` for the gecko code.
/// `larger_address`: Indicates if the given address is >= `0x01000000`.
/// ## Returns
/// `Result<String, GeckoCodeConversionError>`
fn from_04(cursor: &mut Cursor<&[u32]>, larger_address: bool) -> Result<String, GeckoCodeConversionError> {
    let mut result = "// - Constant 32-bit RAM write -\n".to_string();
    result += &format!("// Target address: 0x{:X}\n", get_code_address(cursor, larger_address));
    result += &format!("// Value: 0x{:X}\n", get_and_seek(cursor));
    Ok(result)
}

/// # 0x80: Set Gecko Register to...
/// ## Parameters
/// `cursor`: The `Cursor` for the gecko code.
/// `larger_address`: Indicates if the given address is >= `0x01000000`.
/// ## Returns
/// `Result<String, GeckoCodeConversionError>`
fn from_80(cursor: &mut Cursor<&[u32]>) -> Result<String, GeckoCodeConversionError> {
    let register = get_and_seek(cursor) & 0x000000FF;
    let value = get_and_seek(cursor);

    Ok(format!("// - Set Gecko Register {register} to 0x{:X}\n", value))
}

/// # 0x82: Load into Gecko Register
/// ## Parameters
/// `cursor`: The `Cursor` for the gecko code.
/// `larger_address`: Indicates if the given address is >= `0x01000000`.
/// ## Returns
/// `Result<String, GeckoCodeConversionError>`
fn from_82(cursor: &mut Cursor<&[u32]>) -> Result<String, GeckoCodeConversionError> {
    let register = get_and_seek(cursor) & 0x000000FF;
    let value = get_and_seek(cursor);

    Ok(format!("// - Load value 0x{:X} into register {register}\n", value))
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
    result += &format!("// Target address: 0x{:X}\n\n", address);

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
        if right_code == 0 {
            break;
        }

        result += &(ppc::code_to_instruction(right_code) + "\n");
    }

    Ok(result)
}
