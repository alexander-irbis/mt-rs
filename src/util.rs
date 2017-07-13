use std::fmt;
use std::io;
use std::io::Write;

/// Formats byte slice as hex string
pub fn fmt_slice2hex(f: &mut fmt::Formatter, data: &[u8]) -> fmt::Result {
    for x in data {
        write!(f, "{:02x}", x)?;
    }
    Ok(())
}

/// Parses hex string into byte buffer
pub fn hex2buf<W: Write, S: AsRef<[u8]>>(mut buf: W, string: S) -> io::Result<()> {
    let string = string.as_ref();
    if string.is_empty() {
        return Ok(());
    }
    let len = string.len();
    if len % 2 == 1 {
        let d = (string[0] as char).to_digit(16).ok_or(io::Error::new(io::ErrorKind::InvalidData, ""))? as u8;
        buf.write(&[d])?;
    }
    for ch in string[len % 2 .. ].chunks(2) {
        let d0 = (ch[0] as char).to_digit(16).ok_or(io::Error::new(io::ErrorKind::InvalidData, ""))? as u8;
        let d1 = (ch[1] as char).to_digit(16).ok_or(io::Error::new(io::ErrorKind::InvalidData, ""))? as u8;
        buf.write(&[(d0 << 4) + d1])?;
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    static DATA: [u8; 5] = [3, 14, 15, 92, 6];

    struct A;

    impl fmt::Display for A {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            fmt_slice2hex(f, &DATA)
        }
    }

    #[test]
    fn encode_decode() {
        let s = format!("{}", A);
        assert_eq!("030e0f5c06", s);

        let mut buf = [0u8; 5];
        hex2buf(&mut buf[..], &s).unwrap();
        assert_eq!(&DATA, &buf);

        let mut buf = [0u8; 5];
        hex2buf(&mut buf[..], &s[1..]).unwrap();
        assert_eq!(&DATA, &buf);

        let mut buf = Vec::new();
        assert!(hex2buf(&mut buf, "xx").is_err());
    }
}
