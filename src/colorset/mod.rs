//! Colorset

pub mod color;
pub mod name;

use color::{bytes_color_segment, Color, ColorSegment};
use name::{bytes_colorset_name, ColorsetName};
use zerocopy::AsBytes;

use crate::utils::{ClsSize, ExtendBytesMut};
use bytes::{Bytes, BytesMut};
use nom::{
    bytes::complete::take, error::FromExternalError, multi::fold_many0, number::complete::le_u32,
    IResult,
};

use std::ops::Deref;

const CLS_HEADER: [u8; 6] = [0x53, 0x4C, 0x43, 0x43, 0x00, 0x01];

#[derive(Debug, PartialEq)]
pub struct Colorset {
    name: ColorsetName,
    color_segments: ColorSegments,
}

impl Colorset {
    pub fn new() -> Colorset {
        let mut new_colorset_name = ColorsetName::new();
        new_colorset_name.set_str("NewColorset").unwrap();

        Colorset {
            name: new_colorset_name,
            color_segments: ColorSegments::new(),
        }
    }

    pub fn as_bytes(&self) -> Bytes {
        let mut colorset_bytes = BytesMut::with_capacity(self.size_in_cls() as usize);
        self.extend_bytes(&mut colorset_bytes);

        colorset_bytes.freeze()
    }
}

impl ClsSize for Colorset {
    fn size_contents_in_cls(&self) -> u32 {
        6 // color set header 
            + self.name.size_in_cls()
            + 4 // u32 of unknown number
            + self.color_segments.size_in_cls()
    }
}

// serialize
impl ExtendBytesMut for Colorset {
    fn extend_bytes(&self, extended: &mut BytesMut) {
        // extend cls header
        extended.extend_from_slice(&CLS_HEADER);

        // extend colorset name
        self.name.extend_bytes(extended);

        // extend unknown number
        extended.extend_from_slice(4u32.as_bytes());

        // extend color segments
        self.color_segments.extend_bytes(extended);
    }
}

// deserialize
pub fn bytes_colorset(input: &[u8]) -> IResult<&[u8], Colorset> {
    // ignore cls header
    let (input, _) = take(6usize)(input)?;
    // get colorsetName
    let (input, colorset_name) = bytes_colorset_name(input)?;
    // ignore unknow val
    let (input, _) = le_u32(input)?;
    // get color segments
    let (input, color_segments) = bytes_color_segments(input)?;

    let colorset = Colorset {
        name: colorset_name,
        color_segments,
    };
    Ok((input, colorset))
}

#[derive(Debug, PartialEq)]
pub struct ColorSegments {
    val: Vec<ColorSegment>,
}

impl ColorSegments {
    pub fn new() -> Self {
        let mut new_color_segment_vec = Vec::<ColorSegment>::new();
        new_color_segment_vec.push(ColorSegment::new(Color::new(0, 0, 0, true), None));

        ColorSegments {
            val: new_color_segment_vec,
        }
    }
}

impl ClsSize for ColorSegments {
    fn size_in_cls(&self) -> u32 {
        4 // u32 of number of colorsegments
        + 4 // u32 of color segments byte size
        + self.size_contents_in_cls()
    }

    fn size_contents_in_cls(&self) -> u32 {
        self.val
            .iter()
            .fold(0, |acc, item| acc + item.size_in_cls())
    }
}

// serialize
impl ExtendBytesMut for ColorSegments {
    fn extend_bytes(&self, extended: &mut BytesMut) {
        // extend number of colors
        let num_colors = self.len() as u32;
        extended.extend_from_slice(num_colors.as_bytes());

        // extend color segments bytes size
        extended.extend_from_slice(self.size_contents_in_cls().as_bytes());

        // extend color segments
        self.iter().for_each(|cs| cs.extend_bytes(extended));
    }
}

// deserialize
pub fn bytes_color_segments(input: &[u8]) -> IResult<&[u8], ColorSegments> {
    use nom::{error::Error, error::ErrorKind::Fail, Err::Failure};
    // get number of colors
    let (input, num_colors) = le_u32(input)?;
    // ignore color segments bytes
    let (input, _) = le_u32(input)?;
    // get colorsegments
    let (input, color_segment_vec) = fold_many0(
        bytes_color_segment,
        Vec::new,
        |mut acc: Vec<ColorSegment>, item| {
            acc.push(item);
            acc
        },
    )(input)?;

    if color_segment_vec.is_empty() {
        return Err(Failure(Error::from_external_error(
            input,
            Fail,
            "Color segments is empty!", // Colorsegments Error
        )));
    } else if color_segment_vec.len() as u32 != num_colors {
        return Err(Failure(Error::from_external_error(
            input,
            Fail,
            "Error in read color segments", // Colorsegments Error
        )));
    }
    Ok((
        input,
        ColorSegments {
            val: color_segment_vec,
        },
    ))
}

impl Deref for ColorSegments {
    type Target = Vec<ColorSegment>;
    fn deref(&self) -> &Self::Target {
        &self.val
    }
}
