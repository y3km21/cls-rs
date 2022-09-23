//! ColorSegments

pub mod color_segment;

use color_segment::{color, color_name};

use crate::colorset::common;
use bytes;
use nom;
use serde;
use std::{error, fmt, ops};
use zerocopy::AsBytes;

/// ColorSegments
///
///
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct ColorSegments {
    val: Vec<color_segment::ColorSegment>,
}

impl ColorSegments {
    pub fn new() -> Self {
        let mut new_color_segment_vec = Vec::<color_segment::ColorSegment>::new();
        new_color_segment_vec.push(color_segment::ColorSegment::new(
            color::Color::new(0, 0, 0, true),
            Some(color_name::ColorName::with_str("Color0").unwrap()),
        ));

        ColorSegments {
            val: new_color_segment_vec,
        }
    }

    pub fn remove(
        &mut self,
        index: usize,
    ) -> Result<color_segment::ColorSegment, ColorSegmentsError> {
        if index < self.val.len() {
            Ok(self.val.remove(index))
        } else {
            Err(ColorSegmentsError::RemoveIndexError)
        }
    }

    pub fn push(&mut self, color_segment: color_segment::ColorSegment) {
        self.val.push(color_segment)
    }
}

impl common::ClsSize for ColorSegments {
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
impl common::ExtendBytesMut for ColorSegments {
    fn extend_bytes(&self, extended: &mut bytes::BytesMut) {
        use common::ClsSize;
        // extend number of colors
        let num_colors = self.len() as u32;
        extended.extend_from_slice(num_colors.as_bytes());

        // extend color segments bytes size
        extended.extend_from_slice(self.size_contents_in_cls().as_bytes());

        // extend color segments
        self.iter().for_each(|cs| cs.extend_bytes(extended));
    }
}

impl common::TryFromBytes for ColorSegments {
    fn try_from_bytes(input: &[u8]) -> nom::IResult<&[u8], Self>
    where
        Self: Sized,
    {
        use nom::multi::fold_many0;
        use nom::number::complete::le_u32;
        use nom::{error::Error, error::ErrorKind::Fail, error::FromExternalError, Err::Failure};
        // get number of colors
        let (input, num_colors) = le_u32(input)?;
        // ignore color segments bytes
        let (input, _) = le_u32(input)?;
        // get colorsegments
        let (input, color_segment_vec) = fold_many0(
            color_segment::ColorSegment::try_from_bytes,
            Vec::new,
            |mut acc: Vec<color_segment::ColorSegment>, item| {
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
}

impl ops::Deref for ColorSegments {
    type Target = Vec<color_segment::ColorSegment>;
    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

impl ops::DerefMut for ColorSegments {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}

#[derive(Debug)]
pub enum ColorSegmentsError {
    RemoveIndexError,
}

impl fmt::Display for ColorSegmentsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", {
            use ColorSegmentsError::*;
            match self {
                RemoveIndexError => "Invalid Index, cannot remove.",
            }
        })
    }
}

impl error::Error for ColorSegmentsError {}
