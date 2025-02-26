// use ppc750cl as disasm;
use ppc750cl_asm as asm;

fn find_arg_count(mnemonic: &str) -> Option<usize> {
    for (m, c) in EXPECTED_ARG_COUNTS {
        if mnemonic == m {
            return Some(c);
        }
    }

    None
}

fn token_to_numeric_argument(mut token: &str) -> Option<i16> {
    // general purpose registers
    if let Some(num) = token.strip_prefix('r') {
        return num.parse::<i16>().ok(); // GPRs (rX -> X)
    }

    // floating-point registers
    if let Some(num) = token.strip_prefix('f') {
        return num.parse::<i16>().ok(); // FPRs (fX -> X)
    }

    // immediate values
    let mult = {
        if let Some(stripped) = token.strip_prefix('-') {
            token = stripped;
            -1
        } else {
            1
        }
    };


    // handle hex
    if let Some(hex) = token.strip_prefix("0x") {
        let num = i16::from_str_radix(hex, 16).unwrap();
        return Some(num * mult);
    }

    // handle non-hex
    if let Ok(num) = token.parse::<i16>() {
        return Some(num * mult);
    }

    None
}

fn token_to_argument(mut token: &str) -> Option<asm::Argument> {
    // strip parens
    if token.contains('(') {
        token = token.strip_prefix('(')?;
    }

    if token.contains(')') {
        token = token.strip_suffix(')')?;
    }

    // parse
    let arg_value = token_to_numeric_argument(token)?;
    let arg_value = u16::from_ne_bytes(arg_value.to_ne_bytes()) as u32;
    
    Some(asm::Argument::Unsigned(arg_value))
}

fn token_to_arguments(token: &str) -> Option<(asm::Argument, asm::Argument)> {
    if let Some((offset, register)) = offset_to_tokens(token) {
        let offset_arg = token_to_argument(&offset)?;
        let register_arg = token_to_argument(&register)?;

        Some((offset_arg, register_arg))
    } else {
        None
    }
}

fn is_offset(token: &str) -> Option<bool> {
    // check if there are parentheses
    let left_found = token.contains('(');
    // specify "ends with" because it can be malformed by adding characters after it
    let right_found = token.ends_with(')');

    if left_found && right_found {
        // valid parens
        Some(true)
    } else if !(left_found && right_found) {
        // no parens
        Some(false)
    } else {
        // invalid parens
        None
    }
}

fn offset_to_tokens(token: &str) -> Option<(String, String)> {
    let left_paren_pos = token.find('(')?;

    let offset = token[0..left_paren_pos].to_string();
    let register = token[left_paren_pos..].to_string();

    Some((offset, register))
}

fn instr_to_code(line: &str) -> Option<u32> {
    // split into individual tokens
    let mut tokens = line.split([' ', ',']).collect::<Vec<&str>>();

    // get rid of empty lines
    tokens.retain(|t| !t.is_empty());

    // must contain valid tokens
    if tokens.is_empty() {
        return None;
    }
    
    // validate mnemonic and argument count
    let mnemonic = tokens.remove(0);

    // check if this is an instruction with no arguments
    if tokens.len() == 0 {
        if let Ok(assembled) = asm::assemble(mnemonic, &[asm::Argument::None; 5]) {
            return Some(assembled);
        }

        return None;
    }

    let arg_count = find_arg_count(mnemonic)?;
    let found_arg_count = tokens.len() + {
        let mut additional = 0;
        for token in tokens.iter() {
            if let Some(b) = is_offset(token) {
                if b {
                    additional += 1;
                }
            }
        }
        additional
    };

    // tokens now only contains args
    if found_arg_count != arg_count {
        return None;
    }

    // parse arguments
    let mut passed_args = [asm::Argument::None; 5];
    let mut used_args = 0;

    for token in tokens {
        // get the numeric value of the argument
        // i.e. r3 -> 3, f1 -> 1, 0x10 -> 0x10

        if let Some(b) = is_offset(token)  {
            if b {
                // is an offset
                println!("is an offset");

                let args = token_to_arguments(token)?;
                passed_args[used_args] = args.0;
                passed_args[used_args + 1] = args.1;
                used_args += 2;
                continue;
            }
        }

        // is not an offset

        let arg = token_to_argument(token)?;
        passed_args[used_args] = arg;
        used_args += 1;
        

        if used_args > 5 {
            return None;
        }
    }

    if let Ok(assembled) = asm::assemble(mnemonic, &passed_args) {
        return Some(assembled);
    }
    
    None
}

fn main() {
    let instruction = "lwz r3, 0x4(r3)";
    let code = instr_to_code(instruction).unwrap_or_else(|| 0);
    println!("instruction: {instruction}, code: {:X}", code);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_to_code() {
        assert_eq!(0x80630004, instr_to_code("lwz r3, 0x4(r3)").unwrap());
    }
}
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