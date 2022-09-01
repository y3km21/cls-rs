//! Color Segment
//!
//!
//!

use crate::utils::{ClsSize, ExtendBytesMut};
use bytemuck::try_cast_slice;
use bytes::BytesMut;
use nom::{
    bytes::complete::take,
    error::FromExternalError,
    number::complete::{le_u16, le_u32, le_u8},
    sequence::tuple,
    IResult,
};
use serde::Serialize;
use std::{cmp::PartialEq, error::Error, fmt::Display, ops::Deref};
use zerocopy::AsBytes;

/// ColorSegment
#[derive(Debug, Clone, Serialize)]
pub struct ColorSegment {
    color: Color,
    color_name: Option<ColorName>,
}

impl ColorSegment {
    pub fn new(color: Color, color_name: Option<ColorName>) -> Self {
        ColorSegment { color, color_name }
    }

    pub fn with_val(
        red: u8,
        green: u8,
        blue: u8,
        transparency: bool,
        color_name: Option<&str>,
    ) -> Result<Self, ColorNameError> {
        let color = Color::new(red, green, blue, transparency);

        match color_name {
            None => Ok(ColorSegment::new(color, None)),
            Some(val) => ColorName::with_str(val).map(|cn| ColorSegment::new(color, Some(cn))),
        }
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

impl ClsSize for ColorSegment {
    fn size_in_cls(&self) -> u32 {
        4 + self.size_contents_in_cls()
    }

    fn size_contents_in_cls(&self) -> u32 {
        let color_size = self.color.size_in_cls();
        let color_name_exists_flag_size = 4u32; // u32
        let color_name_size = if let Some(color_name) = self.color_name.as_ref() {
            color_name.size_in_cls()
        } else {
            0
        };

        color_size + color_name_exists_flag_size + color_name_size
    }
}

impl ExtendBytesMut for ColorSegment {
    fn extend_bytes(&self, extended: &mut BytesMut) {
        // Extend Size Header
        extended.extend_from_slice(self.size_contents_in_cls().as_bytes());

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

/// Deserialize ColorSegment from &\[u8]
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
///
///
///
///
/// RGB + Transparency
#[derive(Debug, PartialEq, Clone, Serialize)]
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

impl ClsSize for Color {
    fn size_contents_in_cls(&self) -> u32 {
        4
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

/// Deserialize Color from &\[u8]
pub fn bytes_color(input: &[u8]) -> IResult<&[u8], Color> {
    let (input, (red, green, blue, tp)) = tuple((le_u8, le_u8, le_u8, le_u8))(input)?;

    if tp == 0 {
        Ok((input, Color::new(0, 0, 0, true)))
    } else {
        Ok((input, Color::new(red, green, blue, false)))
    }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
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
}

impl ClsSize for ColorName {
    fn size_in_cls(&self) -> u32 {
        2 + self.size_contents_in_cls()
    }

    fn size_contents_in_cls(&self) -> u32 {
        self.bytes_len_utf16 as u32
    }
}

// Serialize ColorName
impl ExtendBytesMut for ColorName {
    fn extend_bytes(&self, extended: &mut BytesMut) {
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

/// Deserialize ColorName from &\[u8]
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
        &self.val
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
