//! Colorset Name
//!
//!

use crate::colorset::common;
use bytes;
use encoding_rs as enc;
use nom;
use serde;
use std::{error, fmt, ops};
use zerocopy::AsBytes;

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct ColorsetName {
    val: String,
}

impl ColorsetName {
    pub fn new() -> Self {
        ColorsetName { val: String::new() }
    }

    /// Set ColorsetName from str
    ///
    /// # Note
    /// ColorsetName has the following restrictions
    ///  - Must be a utf8 string of 192bytes or less.
    ///  - The max number of chars is 64 for 3bytes or less chars.
    ///  - A 4bytes char is counted as two 3bytes or less chars.
    pub fn set_str(&mut self, val: &str) -> Result<(), ColorsetNameError> {
        use ColorsetNameError::*;
        // bytes check
        if val.len() > 192 {
            return Err(StringOver192Bytes);
        }

        // char count check
        let count = val.chars().into_iter().fold(0usize, |cn, c| {
            cn + match c.len_utf8() {
                4 => 2,
                _ => 1,
            }
        });
        if count > 64 {
            return Err(CharCountExceeded64);
        }

        self.val = val.to_owned();
        Ok(())
    }

    /// Encode utf8 to sjis
    ///
    /// # Note
    /// This method conforms to the cls file specification.
    ///     - 4 bytes unmappable utf8 char is converted to "2 whitespace"(0x20,0x20).
    ///     - Less than 4 bytes unmappable utf8 char is converted to "whitespace"(0x20).
    fn encode_sjis(&self) -> Vec<u8> {
        use enc::EncoderResult::*;

        //println!("input - {}", str);
        let mut encoder = enc::SHIFT_JIS.new_encoder();

        let chars_count = self.val.chars().count();
        let mut sjis_buf: Vec<u8> = Vec::with_capacity(chars_count * 2);

        let mut input_str = self.val.as_ref();

        loop {
            let (enc_res, offset) =
                encoder.encode_from_utf8_to_vec_without_replacement(input_str, &mut sjis_buf, true);

            match enc_res {
                InputEmpty => break,
                OutputFull => break, // unreachable,
                Unmappable(c) => {
                    match c.len_utf8() {
                        // Conforms to the cls file specification.
                        4 => {
                            for _ in 0..2 {
                                sjis_buf.push(0x20);
                            }
                        }

                        _ => {
                            sjis_buf.push(0x20);
                        }
                    }
                    //println!("{:?} : {}", enc_res, offset);
                    input_str = &input_str[offset..]
                }
            }
        }

        //println!("output - {:02x?}", sjis_buf);
        sjis_buf
    }
}

impl common::ClsSize for ColorsetName {
    fn size_in_cls(&self) -> u32 {
        4 + self.size_contents_in_cls()
    }

    fn size_contents_in_cls(&self) -> u32 {
        // sjis
        let sjis_buf_size = self.encode_sjis().len() as u32;
        // utf8
        let uf8_buf_size = self.as_bytes().len() as u32;

        8 + sjis_buf_size + uf8_buf_size
    }
}

impl common::ExtendBytesMut for ColorsetName {
    fn extend_bytes(&self, extended: &mut bytes::BytesMut) {
        // sjis
        let sjis_buf = self.encode_sjis();
        let sjis_buf_size = sjis_buf.len() as u16;

        // utf8
        let utf8_buf = self.as_bytes();
        let utf8_buf_size = self.len() as u16;

        // ColorsetName bytesize header
        let bytesize_header: u32 = 8 + sjis_buf_size as u32 + utf8_buf_size as u32;

        // extend bytesize header
        extended.extend_from_slice(bytesize_header.as_bytes());

        // extend sjis
        extended.extend_from_slice(sjis_buf_size.as_bytes());
        extended.extend_from_slice(&sjis_buf);

        // delimiter?
        extended.extend_from_slice(0u32.as_bytes());

        // extend utf8
        extended.extend_from_slice(utf8_buf_size.as_bytes());
        extended.extend_from_slice(utf8_buf);
    }
}

impl common::TryFromBytes for ColorsetName {
    fn try_from_bytes(input: &[u8]) -> nom::IResult<&[u8], Self>
    where
        Self: Sized,
    {
        use nom::bytes::complete::take;
        use nom::number::complete::{le_u16, le_u32};
        use nom::{error::Error, error::ErrorKind::Fail, error::FromExternalError, Err::Failure};
        // get colorsetname bytesize header
        let (input, _) = le_u32(input)?;

        // get sjis name bytesize
        let (input, sjis_bytes_size) = le_u16(input)?;
        // ignore sjis bytes
        let (input, _) = take(sjis_bytes_size as usize)(input)?;
        // ignore delimiter
        let (input, _) = le_u32(input)?;
        // get utf8 name bytesize
        let (input, utf8_bytes_size) = le_u16(input)?;
        // get utf8 bytes
        let (input, utf8_bytes) = take(utf8_bytes_size as usize)(input)?;

        // conv string
        let colorset_name_str = String::from_utf8(utf8_bytes.to_owned())
            .map_err(|err| Failure(Error::from_external_error(input, Fail, err)))?;

        // make ColorsetName
        let mut colorset_name = ColorsetName::new();
        colorset_name
            .set_str(&colorset_name_str)
            .map_err(|err| Failure(Error::from_external_error(input, Fail, err)))?;

        Ok((input, colorset_name))
    }
}

impl ops::Deref for ColorsetName {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

#[derive(Debug)]
pub enum ColorsetNameError {
    StringOver192Bytes,
    CharCountExceeded64,
}

impl fmt::Display for ColorsetNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", {
            use ColorsetNameError::*;
            match self {
                StringOver192Bytes => "String is over 192 bytes.",
                CharCountExceeded64 => {
                    "Char count is exceeded 64.(A 4bytes utf8 char is counted 2 char.)"
                }
            }
        })
    }
}

impl error::Error for ColorsetNameError {}

#[cfg(test)]
mod tests {
    use super::bytes;
    use super::common::*;
    use super::ColorsetName;

    #[test]
    fn name_test() {
        let str0 = "\u{3400}test\u{1f5ff}set\u{0414}";
        let str1 = "\u{1f5ff}testset";
        let str3 = "ßtest\u{3400}";
        let str4 = std::iter::repeat("あ").take(64).collect::<String>();
        let mut str5 = std::iter::repeat("t").take(62).collect::<String>();
        str5.push('\u{1f5ff}');

        [str0, str1, str3, &str4, &str5]
            .into_iter()
            .map(|str| {
                let mut csn = ColorsetName::new();
                csn.set_str(str).unwrap();
                csn
            })
            .for_each(|csn| {
                let mut byte_csn = bytes::BytesMut::new();
                csn.extend_bytes(&mut byte_csn);

                let (_, de_csn) = ColorsetName::try_from_bytes(byte_csn.as_ref()).unwrap();

                assert_eq!(de_csn, csn);
            });
    }
}
