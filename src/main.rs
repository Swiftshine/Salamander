mod ppc;

fn main() -> Result<(), ppc::LineConversionError> {
    let code = ppc::instruction_to_code("stw r5, 0x0(r3)")?;
    println!("{:X}", code);

    Ok(())
}