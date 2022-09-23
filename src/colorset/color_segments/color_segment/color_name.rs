//! ColorName
//!
//! Cls Color Name
//!
//! # Note
//! There are the following restrictions on the strings that can be set for Color Name.
//!     - Within 64chars
//!     - Must be within 128 bytes when converted to utf16le.
//! Be careful when dealing with strings for which surrogate pairs are applied when converted to utf16.
//! In case ClipStudioPaint can no longer satisfy the second constraint above, only the high surrogate will be entered.
//! Try entering 63 characters for 'A' and then 🐙 (the character requiring a surrogate pair).
//!

use crate::colorset::common;
use bytemuck;
use bytes;
use nom;
use serde;
use std::{error, fmt, ops};
use zerocopy::AsBytes;

/// ColorName
///
#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub struct ColorName {
    val: String,
    bytes_len_utf16: u16,
}

impl ColorName {
    pub fn new() -> Self {
        ColorName {
            val: String::new(),
            bytes_len_utf16: 0,
        }
    }

    pub fn with_str(val: &str) -> Result<Self, ColorNameError> {
        let mut cn = Self::new();
        cn.set_str(val)?;

        Ok(cn)
    }

    pub fn set_str(&mut self, val: &str) -> Result<(), ColorNameError> {
        let enc_utf16 = val.encode_utf16();
        let bytes_len_utf16 = enc_utf16.count() * 2;

        if bytes_len_utf16 > 128 {
            Err(ColorNameError::EncodedStringOver128Bytes)
        } else {
            self.val = val.to_owned();
            self.bytes_len_utf16 = bytes_len_utf16 as u16;
            Ok(())
        }
    }

    pub fn validate_str(val: &str) -> Result<(), ColorNameError> {
        let enc_utf16 = val.encode_utf16();
        let bytes_len_utf16 = enc_utf16.count() * 2;

        if bytes_len_utf16 > 128 {
            Err(ColorNameError::EncodedStringOver128Bytes)
        } else {
            Ok(())
        }
    }
}

impl common::ClsSize for ColorName {
    fn size_in_cls(&self) -> u32 {
        2 + self.size_contents_in_cls()
    }

    fn size_contents_in_cls(&self) -> u32 {
        self.bytes_len_utf16 as u32
    }
}

// ColorName into Cls bytes.
impl common::ExtendBytesMut for ColorName {
    fn extend_bytes(&self, extended: &mut bytes::BytesMut) {
        // Extend bytesize header
        extended.extend_from_slice(&self.bytes_len_utf16.as_bytes());

        // Color Name(utf16le)
        let utf16_bytes_iter = self
            .val
            .encode_utf16()
            .map(|utf16| utf16.to_le_bytes())
            .flatten();

        // extend utf16_bytes
        extended.extend(utf16_bytes_iter);
    }
}

impl common::TryFromBytes for ColorName {
    fn try_from_bytes(input: &[u8]) -> nom::IResult<&[u8], Self>
    where
        Self: Sized,
    {
        use nom::bytes::complete::take;
        use nom::number::complete::le_u16;
        use nom::{
            error::ErrorKind::Fail,
            error::{Error, FromExternalError},
            Err::Failure,
        };

        let (input, color_name_size) = le_u16(input)?;

        let (input, color_name_bytes) = take(color_name_size as usize)(input)?;
        let color_name_u16_slice = bytemuck::try_cast_slice::<u8, u16>(color_name_bytes)
            .map_err(|err| Failure(Error::from_external_error(input, Fail, err)))?;
        let color_name_str = String::from_utf16(color_name_u16_slice)
            .map_err(|err| Failure(Error::from_external_error(input, Fail, err)))?;

        let mut color_name = ColorName::new();
        color_name
            .set_str(&color_name_str)
            .map_err(|err| Failure(Error::from_external_error(input, Fail, err)))?;

        Ok((input, color_name))
    }
}

impl ops::Deref for ColorName {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

#[derive(Debug)]
pub enum ColorNameError {
    EncodedStringOver128Bytes,
}

impl fmt::Display for ColorNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", {
            use ColorNameError::*;
            match self {
                EncodedStringOver128Bytes => "Encoded String in utf16 is over 128 bytes.",
            }
        })
    }
}

impl error::Error for ColorNameError {}

#[cfg(test)]
mod tests {

    use super::bytes;
    use super::common::{ExtendBytesMut, TryFromBytes};
    use super::ColorName;

    #[test]
    fn color_name_test() {
        let mut clrnm = ColorName::new();
        let str = "\u{1F5FF}";
        clrnm.set_str(str).unwrap();

        // color name -> bytes
        let mut bytes = bytes::BytesMut::new();
        clrnm.extend_bytes(&mut bytes);

        println!("{:?}", bytes);

        assert_eq!(bytes.as_ref(), &[0x04, 0x00, 0x3D, 0xD8, 0xFF, 0xDD]);

        // bytes -> color name
        let (_, de_clrnm) = ColorName::try_from_bytes(bytes.as_ref()).unwrap();
        assert_eq!(de_clrnm, clrnm);
    }

    #[test]
    fn color_name_error_over128bytes_test() {
        // Exceeds 128 bytes when converted to utf16

        let utf8_char_4byte = "\u{1f5ff}";
        let mut clrnm = ColorName::new();
        assert!(clrnm.set_str(&([utf8_char_4byte; 33].concat())).is_err());

        assert!(clrnm
            .set_str(&(["あ"; 63].concat() + utf8_char_4byte))
            .is_err());

        assert!(clrnm
            .set_str(&(["€"; 63].concat() + utf8_char_4byte))
            .is_err());

        assert!(clrnm
            .set_str(&(["a"; 63].concat() + utf8_char_4byte))
            .is_err());
    }
}
