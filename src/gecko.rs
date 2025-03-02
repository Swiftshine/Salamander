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

#[derive(Error, Debug)]
pub enum GeckoCodeConversionError {
    #[error("Invalid type")]
    InvalidType,

    #[error("Malformed code")]
    Malformed,

    #[error("Empty code")]
    Empty
}

pub fn gecko_code_to_assembly(gecko_code: &[u32]) -> Result<String, GeckoCodeConversionError> {
    // todo - a gecko code can have more than one code type,
    // which means we should process codes until the end of the string


    if gecko_code.len() == 0 {
        return Err(GeckoCodeConversionError::Empty);
    }
    
    if gecko_code.len() % 2 != 0 {
        return Err(GeckoCodeConversionError::Malformed);
    }

    // detect type
    // this is the first byte in the sequence

    let code_type_byte = ((gecko_code[0] & 0xFF000000) >> 0x18) as u8;

    let assembly  = match code_type_byte {
        // Insert Assembly
        0xC2 => {
            process_c2(gecko_code)
        }

        // Invalid/Unspported
        _ => {
            return Err(GeckoCodeConversionError::InvalidType)
        }
    };

    assembly
}

/* Code Types */


/// 0xC2: Insert Assembly
/// A branch to a subroutine containing `code` will
/// be placed at `address`. The code must end with
/// `0x00000000`. If an additional line must be used
/// to do this, use a `nop`, (`0x60000000`).
fn process_c2(gecko_code: &[u32]) -> Result<String, GeckoCodeConversionError> {
    let code_length = gecko_code.len();

    let mut cursor = Cursor::new(gecko_code);
    let mut result = String::new();

    // find address
    let address = gecko_code[0] & 0x00FFFFFF;
    let final_address = 0x80000000 | address;
    result += "// Target address: 0x";
    result += &format!("{:X}\n\n", final_address);

    let _num_lines = gecko_code[1];
    cursor.set_position(cursor.position() + 2);

    // lines
    while (cursor.position() as usize) < code_length {
        let left_code = cursor.get_ref()[cursor.position() as usize];
        let right_code = cursor.get_ref()[cursor.position() as usize + 1];

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

        cursor.set_position(cursor.position() + 2);
    }


    Ok(result)
}