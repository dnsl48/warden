use failure::Error;
use fraction::BigFraction;
use std::fmt::Write;

/// Convert string into a decimal so that it alphabetical ordering
/// is preserved when sorting with other numerics generated from
/// strings.
/// Correct order only expected for ASCII (latin alphabet)
/// The result guaranteed to be normal (not NAN, Infinite nor Zero)
///
/// # Examples
///   - "test" becomes "0.0000000116000000010100000001150000000116"
///   - "тест" becomes "0.0000033489000004654400000332330000033489"
///   - "テスト" becomes "0.000881558700121576670008946659"
pub fn alphabet(value: &str) -> Result<BigFraction, Error> {
    let mut weight = String::with_capacity(value.len() * 4 + 2);
    write!(&mut weight, "0.")?;

    let mut bytes: [u8; 4];

    for char in value.chars() {
        bytes = [0u8; 4];
        char.encode_utf8(&mut bytes);
        write!(&mut weight, "{:010}", u32::from_le_bytes(bytes))?;
    }

    let result = BigFraction::from_decimal_str(&weight)?;

    if !result.is_normal() {
        Err(failure::err_msg(format!("Could not weight the following string \"{}\"", value)))
    } else {
        Ok(result)
    }
}
