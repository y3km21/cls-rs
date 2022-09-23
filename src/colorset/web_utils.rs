//! Web Utils

use js_sys::Number;
use num_traits::{Num, NumCast};
use std::num::ParseIntError;

/// Cast JS Number to Primitive num type
pub fn cast_js_number<T: Num + NumCast>(js_number: Number) -> Option<T> {
    js_number
        .as_f64()
        .map(|float_64| T::from(float_64))
        .flatten()
}

/// parse hex color string to rgb color
/// expected string format is "#FFFFFF"
pub fn parse_hex_color(hex_color: String) -> Result<(u8, u8, u8), ParseIntError> {
    let red = u8::from_str_radix(&hex_color[1..3], 16)?;
    let green = u8::from_str_radix(&hex_color[3..5], 16)?;
    let blue = u8::from_str_radix(&hex_color[5..7], 16)?;

    Ok((red, green, blue))
}
