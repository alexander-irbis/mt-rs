use std::fmt;

pub fn fmt_slice2hex(f: &mut fmt::Formatter, data: &[u8]) -> fmt::Result {
    for x in data {
        write!(f, "{:02x}", x)?;
    }
    Ok(())
}