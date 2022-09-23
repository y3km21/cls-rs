//! ColorSegment
//!
//!

pub mod color;
pub mod color_name;

use crate::colorset::common;
use bytes;
use nom;
use serde;
use zerocopy::AsBytes;

/// ColorSegment
#[derive(Debug, Clone, serde::Serialize)]
pub struct ColorSegment {
    color: color::Color,
    color_name: Option<color_name::ColorName>,
}

impl ColorSegment {
    pub fn new(color: color::Color, color_name: Option<color_name::ColorName>) -> Self {
        ColorSegment { color, color_name }
    }

    pub fn with_val(
        red: u8,
        green: u8,
        blue: u8,
        transparency: bool,
        color_name: Option<&str>,
    ) -> Result<Self, color_name::ColorNameError> {
        let color = color::Color::new(red, green, blue, transparency);

        match color_name {
            None => Ok(ColorSegment::new(color, None)),
            Some(val) => {
                color_name::ColorName::with_str(val).map(|cn| ColorSegment::new(color, Some(cn)))
            }
        }
    }

    pub fn get_color_mut_ref(&mut self) -> &mut color::Color {
        &mut self.color
    }

    pub fn get_color_name_mut_ref(&mut self) -> &mut Option<color_name::ColorName> {
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

impl common::ClsSize for ColorSegment {
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

impl common::ExtendBytesMut for ColorSegment {
    fn extend_bytes(&self, extended: &mut bytes::BytesMut) {
        use common::ClsSize;
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

impl common::TryFromBytes for ColorSegment {
    fn try_from_bytes(input: &[u8]) -> nom::IResult<&[u8], Self>
    where
        Self: Sized,
    {
        use nom::number::complete::le_u32;

        let (input, _) = le_u32(input)?;
        let (input, color) = color::Color::try_from_bytes(input)?;
        let (input, exists_color_name) = le_u32(input)?;
        if exists_color_name == 1 {
            let (input, color_name) = color_name::ColorName::try_from_bytes(input)?;
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
}

#[cfg(test)]
mod tests {

    use super::bytes;
    use super::color;
    use super::color_name;
    use super::common::*;
    use super::ColorSegment;

    mod setup {
        use super::*;
        pub fn color_setup(transparency: bool) -> color::Color {
            color::Color::new(1, 128, 255, transparency)
        }

        pub fn color_name_setup(name: &str) -> color_name::ColorName {
            let mut clrnm = color_name::ColorName::new();
            clrnm.set_str(name).unwrap();
            clrnm
        }
    }

    #[test]
    fn color_segment_test() {
        use setup::*;

        let color = color_setup(false);
        let color_name = color_name_setup("TESTCOLOR");

        // Color seg
        let color_segment = ColorSegment::new(color.clone(), Some(color_name));
        let mut ex_bytes = bytes::BytesMut::new();
        color_segment.extend_bytes(&mut ex_bytes);
        let (_, de_color_segment) = ColorSegment::try_from_bytes(ex_bytes.as_ref()).unwrap();
        assert_eq!(de_color_segment, color_segment);

        // Color seg noname
        let color_segment_no_name = ColorSegment::new(color.clone(), None);
        let mut ex_bytes = bytes::BytesMut::new();
        color_segment_no_name.extend_bytes(&mut ex_bytes);
        let (_, de_color_segment_no_name) =
            ColorSegment::try_from_bytes(ex_bytes.as_ref()).unwrap();
        assert_eq!(de_color_segment_no_name, color_segment_no_name);
    }
}
