//! Color
//!
//! Cls Color
//!
//! # Note
//! Color structs with transparency of True are always converted to [0x0,0x0,0x0,0xFF] when converted to bytes.
//! This is to match the actual transparency color created in ClipStudioPaint.
//!
//! In this case, if you set an arbitrary color and turn on transparency, the color will be transparent with color information in the color palette.
//! This color will be rendered as transparent, but such a color cannot be created in the regular way.

use crate::colorset::common;
use bytes;
use nom;
use serde::{
    self,
    ser::{SerializeSeq, SerializeStruct},
};
use std::{error, fmt};

#[derive(Debug, PartialEq, Clone)]
pub enum SerializeMode {
    Struct,
    Seq,
    Hex,
    HexWithNumberSign,
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
    serialize_mode: SerializeMode,
}

impl Color {
    pub fn new(red: u8, green: u8, blue: u8, transparency: bool) -> Self {
        Color {
            red,
            green,
            blue,
            transparency,
            serialize_mode: SerializeMode::Struct,
        }
    }

    pub fn new_with_hex_color(
        hex_color: &str,
        transparency: bool,
    ) -> Result<Self, ParseHexColorError> {
        let (red, green, blue) = parse_hex_color(hex_color)?;
        Ok(Color {
            red,
            green,
            blue,
            transparency,
            serialize_mode: SerializeMode::Struct,
        })
    }

    pub fn set_rgb(&mut self, red: u8, green: u8, blue: u8) {
        self.red = red;
        self.green = green;
        self.blue = blue;
    }

    pub fn set_rgb_with_hex_color(&mut self, hex_color: &str) -> Result<(), ParseHexColorError> {
        let (red, green, blue) = parse_hex_color(hex_color)?;
        self.red = red;
        self.green = green;
        self.blue = blue;

        Ok(())
    }

    pub fn get_rgb(&self) -> (u8, u8, u8) {
        (self.red, self.green, self.blue)
    }

    pub fn get_hex_color(&self, number_sign: bool) -> String {
        let red_hex = format!("{:02X?}", self.red);
        let green_hex = format!("{:02X?}", self.green);
        let blue_hex = format!("{:02X?}", self.blue);

        [
            if number_sign { "#" } else { "" },
            &red_hex,
            &green_hex,
            &blue_hex,
        ]
        .concat()
    }

    pub fn set_transparency(&mut self, transparency: bool) {
        self.transparency = transparency
    }

    pub fn get_transparency(&self) -> bool {
        self.transparency
    }

    pub fn set_serialize_mode_struct(&mut self) {
        self.serialize_mode = SerializeMode::Struct;
    }

    pub fn set_serialize_mode_seq(&mut self) {
        self.serialize_mode = SerializeMode::Seq;
    }

    pub fn set_serialize_mode_hex(&mut self) {
        self.serialize_mode = SerializeMode::Hex;
    }

    pub fn set_serialize_mode_hex_with_number_sign(&mut self) {
        self.serialize_mode = SerializeMode::HexWithNumberSign;
    }

    pub fn get_serialize_mode(&self) -> &SerializeMode {
        &self.serialize_mode
    }
}

impl common::ClsSize for Color {
    fn size_contents_in_cls(&self) -> u32 {
        4
    }
}

// Color into Cls bytes.
impl common::ExtendBytesMut for Color {
    fn extend_bytes(&self, extended: &mut bytes::BytesMut) {
        if self.transparency {
            extended.extend_from_slice(&[0, 0, 0, 0]);
        } else {
            extended.extend_from_slice(&[self.red, self.green, self.blue, 0xFF]);
        }
    }
}

// Color try from Bytes.
impl common::TryFromBytes for Color {
    fn try_from_bytes(input: &[u8]) -> nom::IResult<&[u8], Self>
    where
        Self: Sized,
    {
        use nom::number::complete::le_u8;

        let (input, (red, green, blue, tp)) =
            nom::sequence::tuple((le_u8, le_u8, le_u8, le_u8))(input)?;

        if tp == 0 {
            Ok((input, Color::new(0, 0, 0, true)))
        } else {
            Ok((input, Color::new(red, green, blue, false)))
        }
    }
}

fn parse_hex_color(hex_color: &str) -> Result<(u8, u8, u8), ParseHexColorError> {
    // #FFFFFF , FFFFFF, #FFF, FFF　is valid
    let mut hex_color = hex_color;

    let hex_color_len = hex_color.len();
    // Check Number sign(#)
    match hex_color_len {
        4 | 7 => {
            if &hex_color[0..1] != "#" {
                // Invalid Hex Error
                return Err(ParseHexColorError::InvalidHexColorStrError);
            }
            hex_color = &hex_color[1..];
        }
        3 | 6 => { // NoOp
        }
        _ => {
            // Invalid Hex Error
            return Err(ParseHexColorError::InvalidHexColorStrError);
        }
    }

    if hex_color.len() == 3 {
        // Short hand
        let red = u8::from_str_radix(&hex_color[0..1].repeat(2), 16)
            .map_err(|e| ParseHexColorError::ParseIntError(e))?;
        let green = u8::from_str_radix(&hex_color[1..2].repeat(2), 16)
            .map_err(|e| ParseHexColorError::ParseIntError(e))?;
        let blue = u8::from_str_radix(&hex_color[2..3].repeat(2), 16)
            .map_err(|e| ParseHexColorError::ParseIntError(e))?;
        Ok((red, green, blue))
    } else {
        // Normal
        let red = u8::from_str_radix(&hex_color[0..2], 16)
            .map_err(|e| ParseHexColorError::ParseIntError(e))?;
        let green = u8::from_str_radix(&hex_color[2..4], 16)
            .map_err(|e| ParseHexColorError::ParseIntError(e))?;
        let blue = u8::from_str_radix(&hex_color[4..6], 16)
            .map_err(|e| ParseHexColorError::ParseIntError(e))?;
        Ok((red, green, blue))
    }
}

impl serde::Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.serialize_mode {
            SerializeMode::Seq => {
                if self.transparency {
                    let seq = serializer.serialize_seq(None)?;
                    seq.end()
                } else {
                    let mut seq = serializer.serialize_seq(Some(3))?;
                    seq.serialize_element(&self.red)?;
                    seq.serialize_element(&self.green)?;
                    seq.serialize_element(&self.blue)?;
                    seq.end()
                }
            }
            SerializeMode::Hex => {
                if self.transparency {
                    serializer.serialize_str("")
                } else {
                    serializer.serialize_str(&self.get_hex_color(false))
                }
            }
            SerializeMode::HexWithNumberSign => {
                if self.transparency {
                    serializer.serialize_str("")
                } else {
                    serializer.serialize_str(&self.get_hex_color(true))
                }
            }
            SerializeMode::Struct => {
                let mut color = serializer.serialize_struct("Color", 4)?;
                color.serialize_field("red", &self.red)?;
                color.serialize_field("green", &self.green)?;
                color.serialize_field("blue", &self.blue)?;
                color.serialize_field("transparency", &self.transparency)?;
                color.end()
            }
        }
    }
}

#[derive(Debug)]
pub enum ParseHexColorError {
    InvalidHexColorStrError,
    ParseIntError(std::num::ParseIntError),
}

impl fmt::Display for ParseHexColorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ParseHexColorError::*;
        match self {
            InvalidHexColorStrError => write!(f, "{}", "Invalid Hex Color Str."),
            ParseIntError(parse_int_error) => write!(f, "{}", parse_int_error),
        }
    }
}

impl error::Error for ParseHexColorError {}

#[cfg(test)]
mod tests {

    use super::bytes;
    use super::common::{ExtendBytesMut, TryFromBytes};
    use super::Color;
    mod setup {
        use super::*;
        pub fn color_setup(transparency: bool) -> Color {
            Color::new(1, 128, 255, transparency)
        }
    }

    use self::setup::color_setup;

    #[test]
    fn rgb_test() {
        // RGB
        let rgb_clr = color_setup(false);
        let mut rgb_bytes = bytes::BytesMut::new();
        rgb_clr.extend_bytes(&mut rgb_bytes);
        assert_eq!(rgb_bytes.as_ref(), [1, 128, 255, 0xFF]);

        let (_, de_rgb_clr) = Color::try_from_bytes(rgb_bytes.as_ref()).unwrap();
        assert_eq!(rgb_clr, de_rgb_clr);
    }

    #[test]
    fn transparency_test() {
        // Transparency
        let mut tp_clr = color_setup(true);
        let mut tp_bytes = bytes::BytesMut::new();
        tp_clr.extend_bytes(&mut tp_bytes);
        assert_eq!(tp_bytes.as_ref(), [0, 0, 0, 0]);

        let (_, de_tp_clr) = Color::try_from_bytes(tp_bytes.as_ref()).unwrap();
        assert_ne!(de_tp_clr, tp_clr);

        // change to expected val
        tp_clr.set_rgb(0, 0, 0);

        assert_eq!(de_tp_clr, tp_clr);
    }

    #[test]
    fn hex_color_test() {
        // Hex Color
        let mut rgb_clr = color_setup(false);

        assert_eq!(&rgb_clr.get_hex_color(true), "#0180FF");
        assert_eq!(&rgb_clr.get_hex_color(false), "0180FF");

        assert!(rgb_clr.set_rgb_with_hex_color("#808080").is_ok());
        assert!(rgb_clr.set_rgb_with_hex_color("ffeedd").is_ok());
        assert!(rgb_clr.set_rgb_with_hex_color("okokok").is_err());

        let new_rgb_clr = Color::new_with_hex_color("#FFeedd", false);
        assert!(new_rgb_clr.is_ok());
        assert_eq!(new_rgb_clr.unwrap(), rgb_clr);
    }

    use super::parse_hex_color;
    #[test]
    fn parse_hex_color_test() {
        let valid_str_arr = ["#FFEE00", "FFEE00", "#FE0", "FE0"];

        for vs in valid_str_arr {
            let parsed = parse_hex_color(vs);
            assert!(
                parsed.is_ok(),
                "{} is expected Valid : {}",
                vs,
                parsed.err().unwrap()
            )
        }

        let invalid_str_arr = ["#FFEEGG", "AFFEE00", "#FE*", "#", "FFEE00#"];

        for invs in invalid_str_arr {
            assert!(
                parse_hex_color(invs).is_err(),
                "{} is expected Invalid",
                invs
            )
        }
    }

    #[test]
    fn serialize_test() {
        use serde_json;

        // struct
        let test_color = Color::new_with_hex_color("#FF8000", false).unwrap();
        let tc_struct_json = serde_json::to_string(&test_color).unwrap();
        assert_eq!(
            tc_struct_json,
            "{\"red\":255,\"green\":128,\"blue\":0,\"transparency\":false}"
        );

        // seq
        let mut test_color = Color::new_with_hex_color("#FF8000", false).unwrap();
        test_color.set_serialize_mode_seq();
        let tc_seq_json = serde_json::to_string(&test_color).unwrap();
        assert_eq!(tc_seq_json, "[255,128,0]");
        // transparency on
        test_color.set_transparency(true);
        let tc_seq_tp_json = serde_json::to_string(&test_color).unwrap();
        assert_eq!(tc_seq_tp_json, "[]");

        // hex
        let mut test_color = Color::new_with_hex_color("#FF8000", false).unwrap();
        test_color.set_serialize_mode_hex();
        let tc_hex_json = serde_json::to_string(&test_color).unwrap();
        assert_eq!(tc_hex_json, "\"FF8000\"");
        // transparecy on
        test_color.set_transparency(true);
        let tc_hex_tp_json = serde_json::to_string(&test_color).unwrap();
        assert_eq!(tc_hex_tp_json, "\"\"");

        // hex with number sign
        let mut test_color = Color::new_with_hex_color("#FF8000", false).unwrap();
        test_color.set_serialize_mode_hex_with_number_sign();
        let tc_hex_with_ns_json = serde_json::to_string(&test_color).unwrap();
        assert_eq!(tc_hex_with_ns_json, "\"#FF8000\"");
        // transparency on
        test_color.set_transparency(true);
        let tc_hex_with_ns_tp_json = serde_json::to_string(&test_color).unwrap();
        assert_eq!(tc_hex_with_ns_tp_json, "\"\"");
    }
}
