use ppc750cl as disasm;
use ppc750cl_asm as asm;
use thiserror::Error;

/// A list of mnemonics and their argument counts.
const EXPECTED_ARG_COUNTS: [(&str, usize); 296] = [
    ("add", 3),
    ("addc", 3),
    ("adde", 3),
    ("addi", 3),
    ("addic", 3),
    ("addic_", 3),
    ("addis", 3),
    ("addme", 2),
    ("addze", 2),
    ("and", 3),
    ("andc", 3),
    ("andi_", 3),
    ("andis_", 3),
    ("b", 1),
    ("bc", 3),
    ("bcctr", 2),
    ("bclr", 2),
    ("cmp", 4),
    ("cmpi", 4),
    ("cmpl", 4),
    ("cmpli", 4),
    ("cntlzw", 2),
    ("crand", 3),
    ("crandc", 3),
    ("creqv", 3),
    ("crnand", 3),
    ("crnor", 3),
    ("cror", 3),
    ("crorc", 3),
    ("crxor", 3),
    ("dcbf", 2),
    ("dcbi", 2),
    ("dcbst", 2),
    ("dcbt", 2),
    ("dcbtst", 2),
    ("dcbz", 2),
    ("dcbz_l", 2),
    ("divw", 3),
    ("divwu", 3),
    ("eciwx", 3),
    ("ecowx", 3),
    ("eieio", 0),
    ("eqv", 3),
    ("extsb", 2),
    ("extsh", 2),
    ("fabs", 2),
    ("fadd", 3),
    ("fadds", 3),
    ("fcmpo", 3),
    ("fcmpu", 3),
    ("fctiw", 2),
    ("fctiwz", 2),
    ("fdiv", 3),
    ("fdivs", 3),
    ("fmadd", 4),
    ("fmadds", 4),
    ("fmr", 2),
    ("fmsub", 4),
    ("fmsubs", 4),
    ("fmul", 3),
    ("fmuls", 3),
    ("fnabs", 2),
    ("fneg", 2),
    ("fnmadd", 4),
    ("fnmadds", 4),
    ("fnmsub", 4),
    ("fnmsubs", 4),
    ("fres", 2),
    ("frsp", 2),
    ("frsqrte", 2),
    ("fsel", 4),
    ("fsub", 3),
    ("fsubs", 3),
    ("icbi", 2),
    ("isync", 0),
    ("lbz", 3),
    ("lbzu", 3),
    ("lbzux", 3),
    ("lbzx", 3),
    ("lfd", 3),
    ("lfdu", 3),
    ("lfdux", 3),
    ("lfdx", 3),
    ("lfs", 3),
    ("lfsu", 3),
    ("lfsux", 3),
    ("lfsx", 3),
    ("lha", 3),
    ("lhau", 3),
    ("lhaux", 3),
    ("lhax", 3),
    ("lhbrx", 3),
    ("lhz", 3),
    ("lhzu", 3),
    ("lhzux", 3),
    ("lhzx", 3),
    ("lmw", 3),
    ("lswi", 3),
    ("lswx", 3),
    ("lwarx", 3),
    ("lwbrx", 3),
    ("lwz", 3),
    ("lwzu", 3),
    ("lwzux", 3),
    ("lwzx", 3),
    ("mcrf", 2),
    ("mcrfs", 2),
    ("mcrxr", 1),
    ("mfcr", 1),
    ("mffs", 1),
    ("mfmsr", 1),
    ("mfspr", 2),
    ("mfsr", 2),
    ("mfsrin", 2),
    ("mftb", 2),
    ("mtcrf", 2),
    ("mtfsb0", 1),
    ("mtfsb1", 1),
    ("mtfsf", 2),
    ("mtfsfi", 2),
    ("mtmsr", 1),
    ("mtspr", 2),
    ("mtsr", 2),
    ("mtsrin", 2),
    ("mulhw", 3),
    ("mulhwu", 3),
    ("mulli", 3),
    ("mullw", 3),
    ("nand", 3),
    ("neg", 2),
    ("nor", 3),
    ("or", 3),
    ("orc", 3),
    ("ori", 3),
    ("oris", 3),
    ("psq_l", 5),
    ("psq_lu", 5),
    ("psq_lux", 5),
    ("psq_lx", 5),
    ("psq_st", 5),
    ("psq_stu", 5),
    ("psq_stux", 5),
    ("psq_stx", 5),
    ("ps_abs", 2),
    ("ps_add", 3),
    ("ps_cmpo0", 3),
    ("ps_cmpo1", 3),
    ("ps_cmpu0", 3),
    ("ps_cmpu1", 3),
    ("ps_div", 3),
    ("ps_madd", 4),
    ("ps_madds0", 4),
    ("ps_madds1", 4),
    ("ps_merge00", 3),
    ("ps_merge01", 3),
    ("ps_merge10", 3),
    ("ps_merge11", 3),
    ("ps_mr", 2),
    ("ps_msub", 4),
    ("ps_mul", 3),
    ("ps_muls0", 3),
    ("ps_muls1", 3),
    ("ps_nabs", 2),
    ("ps_neg", 2),
    ("ps_nmadd", 4),
    ("ps_nmsub", 4),
    ("ps_res", 2),
    ("ps_rsqrte", 2),
    ("ps_sel", 4),
    ("ps_sub", 3),
    ("ps_sum0", 4),
    ("ps_sum1", 4),
    ("rfi", 0),
    ("rlwimi", 5),
    ("rlwinm", 5),
    ("rlwnm", 5),
    ("sc", 0),
    ("slw", 3),
    ("sraw", 3),
    ("srawi", 3),
    ("srw", 3),
    ("stb", 3),
    ("stbu", 3),
    ("stbux", 3),
    ("stbx", 3),
    ("stfd", 3),
    ("stfdu", 3),
    ("stfdux", 3),
    ("stfdx", 3),
    ("stfiwx", 3),
    ("stfs", 3),
    ("stfsu", 3),
    ("stfsux", 3),
    ("stfsx", 3),
    ("sth", 3),
    ("sthbrx", 3),
    ("sthu", 3),
    ("sthux", 3),
    ("sthx", 3),
    ("stmw", 3),
    ("stswi", 3),
    ("stswx", 3),
    ("stw", 3),
    ("stwbrx", 3),
    ("stwcx_", 3),
    ("stwu", 3),
    ("stwux", 3),
    ("stwx", 3),
    ("subf", 3),
    ("subfc", 3),
    ("subfe", 3),
    ("subfic", 3),
    ("subfme", 2),
    ("subfze", 2),
    ("sync", 0),
    ("tlbie", 1),
    ("tlbsync", 0),
    ("tw", 3),
    ("twi", 3),
    ("xor", 3),
    ("xori", 3),
    ("xoris", 3),
    ("bctr", 0),
    ("bdnz", 1),
    ("bdnzf", 2),
    ("bdnzflr", 1),
    ("bdnzlr", 0),
    ("bdnzt", 2),
    ("bdnztlr", 1),
    ("bdz", 1),
    ("bdzf", 2),
    ("bdzflr", 1),
    ("bdzlr", 0),
    ("bdzt", 2),
    ("bdztlr", 1),
    ("beq", 0),
    ("blt", 4),
    ("clrlwi", 3),
    ("clrrwi", 3),
    ("cmpd", 1),
    ("crmove", 2),
    ("crnot", 2),
    ("crset", 1),
    ("extlwi", 4),
    ("extrwi", 4),
    ("li", 2),
    ("lis", 2),
    ("mfctr", 1),
    ("mfdar", 1),
    ("mfdbatl", 2),
    ("mfdbatu", 2),
    ("mfdec", 1),
    ("mfdsisr", 1),
    ("mfear", 1),
    ("mfibatl", 2),
    ("mfibatu", 2),
    ("mflr", 1),
    ("mfsdr1", 1),
    ("mfsprg", 2),
    ("mfsrr0", 1),
    ("mfsrr1", 1),
    ("mfxer", 1),
    ("mr", 2),
    ("mtctr", 1),
    ("mtdar", 1),
    ("mtdbatl", 2),
    ("mtdbatu", 2),
    ("mtdec", 1),
    ("mtdsisr", 1),
    ("mtear", 1),
    ("mtibatl", 2),
    ("mtibatu", 2),
    ("mtlr", 1),
    ("mtsdr1", 1),
    ("mtsprg", 2),
    ("mtsrr0", 1),
    ("mtsrr1", 1),
    ("mttbl", 1),
    ("mttbu", 1),
    ("mtxer", 1),
    ("nop", 0),
    ("rotlw", 3),
    ("rotlwi", 3),
    ("rotrwi", 3),
    ("slwi", 3),
    ("srwi", 3),
    ("subi", 3),
    ("subic", 3),
    ("subic_", 3),
    ("subis", 3),
    ("trap", 0),
    ("tweq", 2),
    ("twgti", 2),
    ("twlge", 2),
    ("twllei", 2),
    ("twui", 2)
];

#[derive(Error, Debug)]
pub enum LineConversionError {
    #[error("Malformed parentheses")]
    MalformedParentheses,
    #[error("Incorrect argument count")]
    IncorrectArgCount,
    #[error("Invalid numeric token")]
    InvalidNumericToken,
    // #[error("Invalid token")]
    // InvalidToken,
    #[error("Invalid token count")]
    InvalidTokenCount,
    #[error("Invalid instruction")]
    InvalidInstruction
}

/// Returns the number of arguments needed for a certain instruction.
fn find_arg_count(mnemonic: &str) -> Result<usize, LineConversionError> {
    for (m, c) in EXPECTED_ARG_COUNTS {
        if mnemonic == m {
            return Ok(c);
        }
    }

    Err(LineConversionError::IncorrectArgCount)
}

/// Returns if the token is a offset/register pair
fn is_offset_register_pair(token: &str) -> Result<bool, LineConversionError> {
    // check if there are parentheses
    let left_found = token.contains('(');

    // specify "ends with" because it can be malformed by adding characters after it
    let right_found = token.ends_with(')');

    if left_found && right_found {
        // valid parens
        Ok(true)
    } else if !(left_found && right_found) {
        // no parens
        Ok(false)
    } else {
        // invalid parens
        Err(LineConversionError::MalformedParentheses)
    }
}

/// Converts a token to an i16 numeric argument.
fn token_to_i16(token: &str) -> Result<i16, LineConversionError> {
    // general purpose registers
    if let Some(num) = token.strip_prefix('r') {
        return Ok(num.parse::<i16>().unwrap()); // GPRs (rX -> X)
    }

    // floating-point registers
    if let Some(num) = token.strip_prefix('f') {
        return Ok(num.parse::<i16>().unwrap()); // FPRs (fX -> X)
    }

    let mut parsed_token = token;

    // immediate values
    let mult = {
        if let Some(strip) = token.strip_prefix('-') {
            parsed_token = strip;
            -1
        } else {
            1
        }
    };

    // handle hex
    if let Some(hex) = parsed_token.strip_prefix("0x") {
        let num = i16::from_str_radix(hex, 16).unwrap();
        return Ok(num * mult);
    }

    // handle non-hex
    if let Ok(num) = parsed_token.parse::<i16>() {
        return Ok(num * mult);
    }

    
    Err(LineConversionError::InvalidNumericToken)
}


/// Converts a token to an assembler argument.
fn token_to_assembler_argument(token: &str) -> Result<asm::Argument, LineConversionError> {
    // strip parens

    let token = if token.contains('(') {
        token.strip_prefix('(').unwrap()
    } else {
        token
    };

    let token = if token.contains(')') {
        token.strip_prefix(')').unwrap()
    } else {
        token
    };

    // parse
    let arg_value = token_to_i16(token)?;
    let arg_value = u16::from_ne_bytes(arg_value.to_ne_bytes()) as u32;

    Ok(asm::Argument::Unsigned(arg_value))
}


/// Converts a list of tokens to a list of assembler arguments.
fn tokens_to_assembler_arguments(tokens: &[&str]) -> Result<Vec<asm::Argument>, LineConversionError> {
    let mut args: Vec<asm::Argument> = Vec::new();

    for token in tokens {
        if is_offset_register_pair(token)? {
            // split tokens
            let left_paren_pos = token.find('(').unwrap();
            let offset = &token[0..left_paren_pos];
            let register = &token[left_paren_pos..token.len() - 1];
            
            // add to args
            args.push(token_to_assembler_argument(offset)?);
            args.push(token_to_assembler_argument(register)?);
        } else {
            // not an offset
            args.push(token_to_assembler_argument(token)?);
        }
    }

    Ok(args)
}

/// Converts a written line of PowerPC to a u32.
pub fn instruction_to_code(instr: &str) -> Result<u32, LineConversionError> {
    // split into individual tokens
    let mut tokens = instr.split([' ', ',']).collect::<Vec<&str>>();

    // get rid of empty lines
    tokens.retain(|t| !t.is_empty());

    // must contain valid tokens
    if tokens.is_empty() {
        return Err(LineConversionError::InvalidTokenCount);
    }

    // validate mnemonic and argument count
    let mnemonic = tokens.remove(0);

    // check if this is an instruction with no arguments
    if tokens.len() == 0 {
        if let Ok(assembled) = asm::assemble(mnemonic, &[asm::Argument::None; 5]) {
            return Ok(assembled);
        }

        return Err(LineConversionError::IncorrectArgCount);
    }

    let mut args = tokens_to_assembler_arguments(&tokens)?;

    // validate arg count
    if args.len() != find_arg_count(mnemonic)? {
        return Err(LineConversionError::IncorrectArgCount);
    }

    // ensure the length is 5
    args.resize(5, asm::Argument::None);

    let mut passed_args = [asm::Argument::None; 5];

    for i in 0..5 {
        passed_args[i] = args[i];
    }

    if let Ok(assembled) = asm::assemble(mnemonic, &passed_args) {
        Ok(assembled)
    } else {
        Err(LineConversionError::InvalidInstruction)
    }
}

/// Converts binary PowerPC into a written line.
pub fn code_to_instruction(code: u32) -> String {
    disasm::Ins::new(code).simplified().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_offset_register_pair() -> Result<(), LineConversionError> {
        assert!(is_offset_register_pair("0x4(r3)")?);
        assert!(!is_offset_register_pair("0x10")?);
        assert!(is_offset_register_pair("0x8(r4)")?);
        assert!(!is_offset_register_pair("24")?);
        assert!(is_offset_register_pair("0x10(r5)")?);

        Ok(())
    }

    #[test]
    fn check_numeric_tokens() {
        assert_eq!(9, token_to_i16("9").unwrap());
        assert_eq!(16, token_to_i16("0x10").unwrap());
        assert_eq!(-100, token_to_i16("-100").unwrap());
        assert_eq!(-0x100, token_to_i16("-0x100").unwrap());
    }

    #[test]
    fn check_arguments() {
        let a = tokens_to_assembler_arguments(
            &["r3", "0x4(r3)"]
        ).unwrap();

        assert_eq!(3, a.len());

        let b = [
            asm::Argument::Unsigned(3),
            asm::Argument::Unsigned(4),
            asm::Argument::Unsigned(3)
        ];

        let matching = a.iter()
            .zip(b.iter())
            .filter(|&(arg1, arg2)| arg1 == arg2)
            .count();

        assert_eq!(matching, a.len());
        assert_eq!(matching, b.len());
    }

    #[test]
    fn check_instruction_to_code() -> Result<(), LineConversionError> {
        assert_eq!(
            0x80630004,
            instruction_to_code("lwz r3, 0x4(r3)")?
        );

        assert_eq!(
            0x4E800020,
            instruction_to_code("blr")?
        );

        Ok(())
    }

    #[test]
    fn check_code_to_instruction() {
        assert!("blr".eq(&code_to_instruction(0x4E800020)));
        assert!("lwz r3, 0x4(r3)".eq(&code_to_instruction(0x80630004)));
    }
}