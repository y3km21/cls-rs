//! Colorset
//!
//!

pub mod color_segments;
pub mod colorset_name;
pub mod common;
pub mod web_utils;

use js_sys::{Boolean, JsString, Number};
use zerocopy::AsBytes;

use bytes::{Bytes, BytesMut};
use nom;
use serde;
use web_utils::{cast_js_number, parse_hex_color};

#[cfg(feature = "web")]
use crate::wasm::*;

#[cfg_attr(feature = "web", wasm_bindgen)]
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct Colorset {
    name: colorset_name::ColorsetName,
    color_segments: color_segments::ColorSegments,
}

#[cfg_attr(feature = "web", wasm_bindgen)]
impl Colorset {
    #[cfg_attr(feature = "web", wasm_bindgen(constructor))]
    pub fn new() -> Colorset {
        let mut new_colorset_name = colorset_name::ColorsetName::new();
        new_colorset_name.set_str("NewColorset").unwrap();

        Colorset {
            name: new_colorset_name,
            color_segments: color_segments::ColorSegments::new(),
        }
    }
}

impl Colorset {
    pub fn as_bytes(&self) -> Bytes {
        use common::{ClsSize, ExtendBytesMut};
        let mut colorset_bytes = BytesMut::with_capacity(self.size_in_cls() as usize);
        self.extend_bytes(&mut colorset_bytes);

        colorset_bytes.freeze()
    }
}

impl common::ClsSize for Colorset {
    fn size_contents_in_cls(&self) -> u32 {
        6 // color set header 
            + self.name.size_in_cls()
            + 4 // u32 of unknown number
            + self.color_segments.size_in_cls()
    }
}

/// CLS File Header
const CLS_HEADER: [u8; 6] = [0x53, 0x4C, 0x43, 0x43, 0x00, 0x01];

// serialize
impl common::ExtendBytesMut for Colorset {
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

impl common::TryFromBytes for Colorset {
    fn try_from_bytes(input: &[u8]) -> nom::IResult<&[u8], Self>
    where
        Self: Sized,
    {
        use nom::{bytes::complete::take, number::complete::le_u32};
        // ignore cls header
        let (input, _) = take(6usize)(input)?;
        // get colorsetName
        let (input, colorset_name) = colorset_name::ColorsetName::try_from_bytes(input)?;
        // ignore unknow val
        let (input, _) = le_u32(input)?;
        // get color segments
        let (input, color_segments) = color_segments::ColorSegments::try_from_bytes(input)?;

        let colorset = Colorset {
            name: colorset_name,
            color_segments,
        };
        Ok((input, colorset))
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
                    color_segments::color_segment::color_name::ColorName::with_str(&color_name)
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

        let new_clr =
            color_segments::color_segment::color::Color::new(red, green, blue, transparency);
        let new_clr_name = if color_name.is_empty() {
            None
        } else {
            Some(
                color_segments::color_segment::color_name::ColorName::with_str(&color_name)
                    .map_err(|err| JsValue::from(err.to_string()))?,
            )
        };

        let new_clr_segment =
            color_segments::color_segment::ColorSegment::new(new_clr, new_clr_name);

        self.color_segments.push(new_clr_segment);

        Ok(())
    }

    #[wasm_bindgen(js_name = "validateColorName")]
    pub fn validate_color_name(color_name: JsString) -> Result<(), JsValue> {
        let color_name = color_name
            .as_string()
            .map_or(Err(JsValue::from("Invalid Input String")), |str| Ok(str))?;

        color_segments::color_segment::color_name::ColorName::validate_str(&color_name)
            .map_err(|err| JsValue::from(err.to_string()))
    }
}

#[cfg(feature = "web")]
#[wasm_bindgen(js_name = "withUint8Array")]
pub fn with_uint8_array(arr: Uint8Array) -> Colorset {
    use self::common::TryFromBytes;

    let buf = arr.to_vec();
    let (_, new_cls) = Colorset::try_from_bytes(&buf).unwrap(); // Error 処理入れてどうぞ
    new_cls
}

#[cfg(test)]
mod tests {
    use super::common::*;
    use super::Colorset;

    #[test]
    fn colorset_test() {
        let new_colorset = Colorset::new();

        let cs_b = new_colorset.as_bytes();

        let (_, de_cs) = Colorset::try_from_bytes(cs_b.as_ref()).unwrap();

        assert_eq!(de_cs, new_colorset);
    }
}
