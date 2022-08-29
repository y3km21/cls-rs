use self::common::{color_name_setup, color_setup};
use bytes::BytesMut;
use cls_rs::colorset::color::{
    bytes_color, bytes_color_name, bytes_color_segment, Color, ColorName, ColorSegment,
    ExtendBytesMut,
};

mod common {
    use super::*;
    pub fn color_setup(transparency: bool) -> Color {
        Color::new(81, 82, 83, transparency)
    }

    pub fn color_name_setup(name: &str) -> ColorName {
        let mut clrnm = ColorName::new();
        clrnm.set_str(name).unwrap();
        clrnm
    }
}

#[test]
fn color_test() {
    // RGB
    let rgb_clr = color_setup(false);
    let mut rgb_bytes = BytesMut::new();
    rgb_clr.extend_bytes(&mut rgb_bytes);
    assert_eq!(rgb_bytes.as_ref(), [81, 82, 83, 0xFF]);

    let (_, de_rgb_clr) = bytes_color(rgb_bytes.as_ref()).unwrap();
    assert_eq!(rgb_clr, de_rgb_clr);

    // Transparency
    let mut tp_clr = color_setup(true);
    let mut tp_bytes = BytesMut::new();
    tp_clr.extend_bytes(&mut tp_bytes);
    assert_eq!(tp_bytes.as_ref(), [0, 0, 0, 0]);

    let (_, de_tp_clr) = bytes_color(tp_bytes.as_ref()).unwrap();
    assert_ne!(de_tp_clr, tp_clr);

    // change　to expected val
    tp_clr.set_rgb(0, 0, 0);

    assert_eq!(de_tp_clr, tp_clr);
}

#[test]
fn color_name_test() {
    let mut clrnm = ColorName::new();
    let str = "\u{1F5FF}";
    clrnm.set_str(str).unwrap();

    // color name -> bytes
    let mut bytes = BytesMut::new();
    clrnm.extend_bytes(&mut bytes);

    println!("{:?}", bytes);

    assert_eq!(bytes.as_ref(), &[0x04, 0x00, 0x3D, 0xD8, 0xFF, 0xDD]);

    // bytes -> color name
    let (_, de_clrnm) = bytes_color_name(bytes.as_ref()).unwrap();
    assert_eq!(de_clrnm, clrnm);
}

#[test]
fn color_name_error_test() {
    let utf8_char_4byte = "\u{1f5ff}";

    let mut clrnm = ColorName::new();

    assert!(clrnm.set_str(&([utf8_char_4byte; 33].concat())).is_err());

    assert!(clrnm
        .set_str(&(["あ"; 63].concat() + utf8_char_4byte))
        .is_err());
    assert!(clrnm
        .set_str(&(["€"; 63].concat() + utf8_char_4byte))
        .is_err());
    assert!(clrnm
        .set_str(&(["a"; 63].concat() + utf8_char_4byte))
        .is_err());
}

#[test]
fn color_segment_test() {
    let color = color_setup(false);
    let color_name = color_name_setup("TESTCOLOR");

    // Color seg
    let color_segment = ColorSegment::new(color.clone(), Some(color_name));
    let mut ex_bytes = BytesMut::new();
    color_segment.extend_bytes(&mut ex_bytes);
    let (_, de_color_segment) = bytes_color_segment(ex_bytes.as_ref()).unwrap();
    assert_eq!(de_color_segment, color_segment);

    // Color seg noname
    let color_segment_no_name = ColorSegment::new(color.clone(), None);
    let mut ex_bytes = BytesMut::new();
    color_segment_no_name.extend_bytes(&mut ex_bytes);
    let (_, de_color_segment_no_name) = bytes_color_segment(ex_bytes.as_ref()).unwrap();
    assert_eq!(de_color_segment_no_name, color_segment_no_name);

    //let mut color_segment = color_segment;
    //color_segment
    //    .get_color_name_mut_ref()
    //    .as_mut()
    //    .unwrap()
    //    .set_str("TESTCOLOR\u{1f5ff}")
    //    .unwrap();
}
