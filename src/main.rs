mod ppc;
mod gecko;

use std::fs;
use anyhow::Result;
use gecko::convert_from_gecko_code_values;

fn main() -> Result<()> {
    let gecko_code = fs::read_to_string("sample_codes/sample_code_3.txt")?;

    println!("Gecko Code:\n{gecko_code}\n");

    let words = gecko_code.split([' ', '\n']).collect::<Vec<&str>>();
    
    let mut values: Vec<u32> = Vec::new();

    for word in words {
        values.push(u32::from_str_radix(word, 16)?);
    }

    let assembly = convert_from_gecko_code_values(&values)?;

    println!("{assembly}");
    Ok(())
}