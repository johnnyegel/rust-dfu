/// Parses an input string to a u32 integer. The input string can be either a decimal or hex.
pub fn usize_from_string(instr: &str) -> Result<usize, ()> {
    // If the input string has no chars, return error
    if instr.len() == 0 { 
        return Err(());
    }
    // By default, set the num offset to 0 and radix to 10
    let mut radix = 10;
    let mut remain = &instr[0..];

    // Check for "skippable" start, where 0 sets radix to 8
    if remain.starts_with('0') {
        radix = 8;
    }
    if radix == 8 || remain.starts_with('\\') {
        remain = skip(remain)?;
    }

    // At this point, we only need to check for an x, which means hex, or o which means octal
    if remain.starts_with('x') {
        radix = 16;
        remain = skip(remain)?;
    }
    else if remain.starts_with('o') {
        radix = 8;
        remain = skip(remain)?;
    }
    else if remain.starts_with('b') {
        radix = 2;
        remain = skip(remain)?;
    }
    // Do tha parsing, and don't give a damn about the error. If it fail, it fail
    match usize::from_str_radix(remain, radix) {
        Ok(res) => Ok(res),
        Err(_) => Err(())
    }
}

fn skip(instr: &str) -> Result<&str, ()> {
    // If the stringlength is less than we will skip, return error
    if instr.len() < 2 {
        return Err(())
    }
    Ok(&instr[1..])
}