//! Colorset Name
//!
//!

use crate::utils::{ClsSize, ExtendBytesMut};

use std::{error::Error, fmt::Display, ops::Deref};

use encoding_rs as enc;
use nom::{
    bytes::complete::take,
    error::FromExternalError,
    number::complete::{le_u16, le_u32},
    IResult,
};
use serde::Serialize;
use zerocopy::AsBytes;

#[derive(Debug, PartialEq, Serialize)]
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

impl ClsSize for ColorsetName {
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

impl ExtendBytesMut for ColorsetName {
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

/// Deserialize ColorsetName from &\[u8]
pub fn bytes_colorset_name(input: &[u8]) -> IResult<&[u8], ColorsetName> {
    use nom::{error::Error, error::ErrorKind::Fail, Err::Failure};

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

impl Deref for ColorsetName {
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

impl Display for ColorsetNameError {
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

impl Error for ColorsetNameError {}
