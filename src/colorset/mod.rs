pub mod color;
pub mod name;

use color::ColorSegment;
use name::ColorsetName;

const CLS_HEADER: [u8; 6] = [0x53, 0x4C, 0x43, 0x43, 0x00, 0x01];

pub struct Colorset {
    name: ColorsetName,
    color_segment: ColorSegment,
}


