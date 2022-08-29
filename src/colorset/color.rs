//! Color Segment
//!
//!
//!

pub use crate::utils::ExtendBytesMut;
use bytemuck::try_cast_slice;
use bytes::BytesMut;
use nom::{
    bytes::complete::take,
    error::FromExternalError,
    number::complete::{le_u16, le_u32, le_u8},
    sequence::tuple,
    IResult,
};
use std::cmp::PartialEq;
use std::error::Error;
use std::fmt::Display;
use std::ops::Deref;
use zerocopy::AsBytes;

/// ColorSegment
#[derive(Debug, Clone)]
pub struct ColorSegment {
    color: Color,
    color_name: Option<ColorName>,
}

impl ColorSegment {
    pub fn new(color: Color, color_name: Option<ColorName>) -> Self {
        ColorSegment { color, color_name }
    }

    pub fn get_color_mut_ref(&mut self) -> &mut Color {
        &mut self.color
    }

    pub fn get_color_name_mut_ref(&mut self) -> &mut Option<ColorName> {
        &mut self.color_name
    }
}

impl PartialEq for ColorSegment {
    fn eq(&self, other: &Self) -> bool {
        if self.color == other.color {
            match (self.color_name.as_ref(), other.color_name.as_ref()) {
                (None, None) => true,
                (Some(s_cn), Some(ot_cn)) => s_cn == ot_cn,
                _ => false,
            }
        } else {
            false
        }
    }
}

impl ExtendBytesMut for ColorSegment {
    fn extend_bytes(&self, extended: &mut BytesMut) {
        // Extend Bytes size of Color Segment
        let color_size = 8u32;
        let color_name_size = if let Some(color_name) = self.color_name.as_ref() {
            2 // u16 of color name bytes size
             + color_name.get_bytes_len_utf16() as u32
        } else {
            0
        };
        extended.extend_from_slice((color_size + color_name_size).as_bytes());

        // Extend Color
        self.color.extend_bytes(extended);

        // Extends Flag of ColorName flag and ColorName
        if let Some(color_name) = self.color_name.as_ref() {
            // Flag is color name exists
            extended.extend_from_slice(1u32.as_bytes());

            color_name.extend_bytes(extended);
        } else {
            // Flag is color name no exists
            extended.extend_from_slice(0u32.as_bytes());
        }
    }
}

pub fn bytes_color_segment(input: &[u8]) -> IResult<&[u8], ColorSegment> {
    let (input, _) = le_u32(input)?;
    let (input, color) = bytes_color(input)?;
    let (input, exists_color_name) = le_u32(input)?;
    if exists_color_name == 1 {
        let (input, color_name) = bytes_color_name(input)?;
        Ok((
            input,
            ColorSegment {
                color,
                color_name: Some(color_name),
            },
        ))
    } else {
        Ok((
            input,
            ColorSegment {
                color,
                color_name: None,
            },
        ))
    }
}

/// Color
///
/// RGB + Transparency
#[derive(Debug, PartialEq, Clone)]
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
    transparency: bool,
}

impl Color {
    pub fn new(red: u8, green: u8, blue: u8, transparency: bool) -> Self {
        Color {
            red,
            green,
            blue,
            transparency,
        }
    }

    pub fn set_rgb(&mut self, red: u8, green: u8, blue: u8) {
        self.red = red;
        self.green = green;
        self.blue = blue;
    }

    pub fn get_rgb(&self) -> (u8, u8, u8) {
        (self.red, self.green, self.blue)
    }

    pub fn set_transparency(&mut self, transparency: bool) {
        self.transparency = transparency
    }

    pub fn get_transparency(&self) -> bool {
        self.transparency
    }
}

// Serialize Color
impl ExtendBytesMut for Color {
    fn extend_bytes(&self, extended: &mut BytesMut) {
        if self.transparency {
            extended.extend_from_slice(&[0, 0, 0, 0]);
        } else {
            extended.extend_from_slice(&[self.red, self.green, self.blue, 0xFF]);
        }
    }
}

// Deserialize Color
pub fn bytes_color(input: &[u8]) -> IResult<&[u8], Color> {
    let (input, (red, green, blue, tp)) = tuple((le_u8, le_u8, le_u8, le_u8))(input)?;

    if tp == 0 {
        Ok((input, Color::new(0, 0, 0, true)))
    } else {
        Ok((input, Color::new(red, green, blue, false)))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ColorName {
    str: String,
    bytes_len_utf16: u16,
}

impl ColorName {
    pub fn new() -> Self {
        ColorName {
            str: String::new(),
            bytes_len_utf16: 0,
        }
    }

    pub fn set_str(&mut self, val: &str) -> Result<(), ColorNameError> {
        let enc_utf16 = val.encode_utf16();
        let bytes_len_utf16 = enc_utf16.count() * 2;

        if bytes_len_utf16 > 128 {
            Err(ColorNameError::EncodedStringOver128Bytes)
        } else {
            self.str = val.to_owned();
            self.bytes_len_utf16 = bytes_len_utf16 as u16;
            Ok(())
        }
    }

    pub fn get_bytes_len_utf16(&self) -> u16 {
        self.bytes_len_utf16
    }
}

// Serialize ColorName
impl ExtendBytesMut for ColorName {
    fn extend_bytes(&self, extended: &mut BytesMut) {
        // byte size of Color Name
        extended.extend_from_slice(&self.bytes_len_utf16.as_bytes());

        // Color Name(utf16le)
        let utf16_bytes_iter = self
            .str
            .encode_utf16()
            .map(|utf16| utf16.to_le_bytes())
            .flatten();
        extended.extend(utf16_bytes_iter);
    }
}

// Deserialize ColorName
pub fn bytes_color_name(input: &[u8]) -> IResult<&[u8], ColorName> {
    use nom::{error::Error, error::ErrorKind::Fail, Err::Failure};

    let (input, color_name_size) = le_u16(input)?;

    let (input, color_name_bytes) = take(color_name_size as usize)(input)?;
    let color_name_u16_slice = try_cast_slice::<u8, u16>(color_name_bytes)
        .map_err(|err| Failure(Error::from_external_error(input, Fail, err)))?;
    let color_name_str = String::from_utf16(color_name_u16_slice)
        .map_err(|err| Failure(Error::from_external_error(input, Fail, err)))?;

    let mut color_name = ColorName::new();
    color_name
        .set_str(&color_name_str)
        .map_err(|err| Failure(Error::from_external_error(input, Fail, err)))?;

    Ok((input, color_name))
}

impl Deref for ColorName {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.str
    }
}

#[derive(Debug)]
pub enum ColorNameError {
    EncodedStringOver128Bytes,
}

impl Display for ColorNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", {
            use ColorNameError::*;
            match self {
                EncodedStringOver128Bytes => "Encoded String in utf16 is over 128 bytes.",
            }
        })
    }
}

impl Error for ColorNameError {}
