//! Colorset

pub mod color;
pub mod name;

use color::{bytes_color_segment, Color, ColorSegment};
use js_sys::{Boolean, JsString, Number};
use name::{bytes_colorset_name, ColorsetName};
use zerocopy::AsBytes;

use crate::utils::{cast_js_number, parse_hex_color, ClsSize, ExtendBytesMut};
use bytes::{Bytes, BytesMut};
use nom::{
    bytes::complete::take, error::FromExternalError, multi::fold_many0, number::complete::le_u32,
    IResult,
};

use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt::Display,
    ops::{Deref, DerefMut},
};

const CLS_HEADER: [u8; 6] = [0x53, 0x4C, 0x43, 0x43, 0x00, 0x01];

#[cfg(feature = "web")]
use crate::wasm::*;

use self::color::ColorName;

#[cfg_attr(feature = "web", wasm_bindgen)]
#[derive(Debug, PartialEq, Serialize)]
pub struct Colorset {
    name: ColorsetName,
    color_segments: ColorSegments,
}

#[cfg_attr(feature = "web", wasm_bindgen)]
impl Colorset {
    #[cfg_attr(feature = "web", wasm_bindgen(constructor))]
    pub fn new() -> Colorset {
        let mut new_colorset_name = ColorsetName::new();
        new_colorset_name.set_str("NewColorset").unwrap();

        Colorset {
            name: new_colorset_name,
            color_segments: ColorSegments::new(),
        }
    }
}

/// API for wasm
#[cfg(feature = "web")]
#[wasm_bindgen]
impl Colorset {
    #[wasm_bindgen(js_name = "getUint8Array")]
    pub fn get_uint8_array(&self) -> Uint8Array {
        let colorset_buf = self.as_bytes().to_vec();
        let js_colorset_buf = serde_wasm_bindgen::to_value(&colorset_buf).unwrap();

        Uint8Array::new(&js_colorset_buf)
    }

    #[wasm_bindgen(js_name = "getJSObject")]
    pub fn get_js_object(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap()
    }

    #[wasm_bindgen(js_name = "setColorsetName")]
    pub fn set_colorset_name(&mut self, colorset_name: JsString) -> Result<(), JsValue> {
        let colorset_name = colorset_name.as_string();

        match colorset_name {
            Some(cs_name) if cs_name.is_empty() == true => self.name.set_str("UserColorset"),
            Some(cs_name) => self.name.set_str(&cs_name),

            None => self.name.set_str("UserColorset"),
        }
        .map_err(|err| JsValue::from(err.to_string()))
    }

    #[wasm_bindgen(js_name = "getColorsetName")]
    pub fn get_colorset_name(&self) -> JsString {
        JsString::from(self.name.clone())
    }

    #[wasm_bindgen(js_name = "setColorName")]
    pub fn set_color_name(&mut self, color_name: JsString, idx: Number) -> Result<(), JsValue> {
        let color_name = color_name
            .as_string()
            .map_or(Err(JsValue::from("Invalid Input String")), |str| Ok(str))?;

        let idx = cast_js_number::<usize>(idx)
            .map_or(Err(JsValue::from("Invalid Input number")), |num| Ok(num))?;

        let cs = self.color_segments.get_mut(idx).map_or(
            Err(JsValue::from(format!(
                "{}th colorsegment does not exist",
                idx
            ))),
            |cs| Ok(cs),
        )?;
        let got_opt_c_name = cs.get_color_name_mut_ref();

        if color_name.is_empty() {
            *got_opt_c_name = None;
        } else {
            if let Some(c_name) = got_opt_c_name {
                c_name
                    .set_str(&color_name)
                    .map_err(|err| JsValue::from(err.to_string()))?;
            } else {
                *got_opt_c_name = Some(
                    ColorName::with_str(&color_name)
                        .map_err(|err| JsValue::from(err.to_string()))?,
                );
            }
        }

        Ok(())
    }

    #[wasm_bindgen(js_name = "setColorRGB")]
    pub fn set_color_rgb(&mut self, red: Number, green: Number, blue: Number, idx: Number) {
        let red: u8 = cast_js_number(red).unwrap();
        let green: u8 = cast_js_number(green).unwrap();
        let blue: u8 = cast_js_number(blue).unwrap();
        let idx: usize = cast_js_number(idx).unwrap();

        let cs = self.color_segments.get_mut(idx).unwrap();
        cs.get_color_mut_ref().set_rgb(red, green, blue);
    }

    #[wasm_bindgen(js_name = "setColorHEX")]
    pub fn set_color_hex(&mut self, hex: JsString, idx: Number) -> Result<JsValue, JsValue> {
        let hex_color = hex
            .as_string()
            .map_or(Err(JsValue::from("Invalid Input String")), |str| Ok(str))?;

        let idx = cast_js_number::<usize>(idx)
            .map_or(Err(JsValue::from("Invalid Input Idx Value")), |uidx| {
                Ok(uidx)
            })?;

        let (red, green, blue) =
            parse_hex_color(hex_color).map_err(|err| JsValue::from(err.to_string()))?;

        let cs = self.color_segments.get_mut(idx).map_or(
            Err(JsValue::from(format!("{}th element does not exists.", idx))),
            |cs| Ok(cs),
        )?;

        cs.get_color_mut_ref().set_rgb(red, green, blue);

        serde_wasm_bindgen::to_value(cs.get_color_mut_ref()).map_err(|err| err.into())
    }

    #[wasm_bindgen(js_name = "setColorTransparency")]
    pub fn set_color_transparency(&mut self, transparency: Boolean, idx: Number) {
        let transparency = transparency.as_bool().unwrap();
        let idx: usize = cast_js_number(idx).unwrap();

        let cs = self.color_segments.get_mut(idx).unwrap();
        cs.get_color_mut_ref().set_transparency(transparency);
    }

    #[wasm_bindgen(js_name = "removeColorSegment")]
    pub fn remove_color_segment(&mut self, idx: Number) -> Result<(), JsValue> {
        let idx: usize =
            cast_js_number(idx).map_or(Err(JsValue::from("Invalid Index")), |cs| Ok(cs))?;

        self.color_segments
            .remove(idx)
            .map_err(|err| JsValue::from(err.to_string()))?;

        Ok(())
    }

    #[wasm_bindgen(js_name = "addColorSegment")]
    pub fn add_color_segment(
        &mut self,
        color_name: JsString,
        hex: JsString,
        transparency: Boolean,
    ) -> Result<(), JsValue> {
        let color_name = color_name
            .as_string()
            .map_or(Err(JsValue::from("Invalid Input String")), |str| Ok(str))?;

        let hex_color = hex
            .as_string()
            .map_or(Err(JsValue::from("Invalid Input String")), |str| Ok(str))?;

        let (red, green, blue) =
            parse_hex_color(hex_color).map_err(|err| JsValue::from(err.to_string()))?;

        let transparency = transparency
            .as_bool()
            .map_or(Err(JsValue::from("Invalid Input Boolean")), |tp| Ok(tp))?;

        let new_clr = Color::new(red, green, blue, transparency);
        let new_clr_name = if color_name.is_empty() {
            None
        } else {
            Some(ColorName::with_str(&color_name).map_err(|err| JsValue::from(err.to_string()))?)
        };

        let new_clr_segment = ColorSegment::new(new_clr, new_clr_name);

        self.color_segments.push(new_clr_segment);

        Ok(())
    }

    #[wasm_bindgen(js_name = "validateColorName")]
    pub fn validate_color_name(color_name: JsString) -> Result<(), JsValue> {
        let color_name = color_name
            .as_string()
            .map_or(Err(JsValue::from("Invalid Input String")), |str| Ok(str))?;

        ColorName::validate_str(&color_name).map_err(|err| JsValue::from(err.to_string()))
    }
}

impl Colorset {
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

#[cfg(feature = "web")]
#[wasm_bindgen(js_name = "withUint8Array")]
pub fn with_uint8_array(arr: Uint8Array) -> Colorset {
    let buf = arr.to_vec();
    let (_, new_cls) = bytes_colorset(&buf).unwrap(); // Error 処理入れてどうぞ
    new_cls
}

/// ColorSegment
///
///
///
///
///
#[derive(Debug, PartialEq, Serialize)]
pub struct ColorSegments {
    val: Vec<ColorSegment>,
}

impl ColorSegments {
    pub fn new() -> Self {
        let mut new_color_segment_vec = Vec::<ColorSegment>::new();
        new_color_segment_vec.push(ColorSegment::new(
            Color::new(0, 0, 0, true),
            Some(ColorName::with_str("Color0").unwrap()),
        ));

        ColorSegments {
            val: new_color_segment_vec,
        }
    }

    pub fn remove(&mut self, index: usize) -> Result<ColorSegment, ColorSegmentsError> {
        if index < self.val.len() {
            Ok(self.val.remove(index))
        } else {
            Err(ColorSegmentsError::RemoveIndexError)
        }
    }

    pub fn push(&mut self, color_segment: ColorSegment) {
        self.val.push(color_segment)
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

impl DerefMut for ColorSegments {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}

#[derive(Debug)]
pub enum ColorSegmentsError {
    RemoveIndexError,
}

impl Display for ColorSegmentsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", {
            use ColorSegmentsError::*;
            match self {
                RemoveIndexError => "Invalid Index, cannot remove.",
            }
        })
    }
}

impl Error for ColorSegmentsError {}
