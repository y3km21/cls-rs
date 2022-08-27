use std::io::Read;

use bincode;
use cls_rs::colorset::color::{self, Color, RGB};
use serde::{Deserialize, Serialize};
use serde_bytes::Bytes;

mod common {
    use super::*;
    pub fn rgb_setup() -> (color::RGB, [u8; 4]) {
        return (color::RGB::new(81, 82, 83), [81u8, 82, 83, 255]);
    }

    pub fn tp_setup() -> (color::TransparentColor, [u8; 4]) {
        return (color::TransparentColor::new(), [0, 0, 0, 0u8]);
    }
}

#[test]
fn color_ser() {
    let (rgb, ser_rgb_expectation) = common::rgb_setup();
    let ser_rgb = bincode::serialize(&rgb).unwrap();
    assert_eq!(ser_rgb.as_slice(), &ser_rgb_expectation);

    let (tp, ser_tp_expectation) = common::tp_setup();
    let ser_tp = bincode::serialize(&tp).unwrap();
    assert_eq!(ser_tp.as_slice(), &ser_tp_expectation);
}

#[test]
fn testest() {
    use std::collections::HashMap;
    use std::io;

    use serde_bytes::ByteBuf;

    fn deserialize_bytebufs() -> bincode::Result<()> {
        let example_data = [
            2, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 116, 119, 111, 1, 0, 0, 0,
            3, 0, 0, 0, 0, 0, 0, 0, 111, 110, 101,
        ];

        let map: std::string::String = bincode::deserialize(&example_data[..]).unwrap();

        println!("{:?}", map);

        Ok(())
    }

    deserialize_bytebufs().unwrap();
}

#[test]
fn color_de() {
    let (de_rgb_expectation, rgb_bytes) = common::rgb_setup();
    let ser_rgb = bincode::serialize(&Box::new(de_rgb_expectation)).unwrap();
    println!("{:?}", &ser_rgb);
    //let ser_rgb_2 = bincode::serialize(&Box::new(de_rgb_expectation)).unwrap();
    //println!("{:?}", ser_rgb_2);
    let de_rgb: Box<dyn Color> =
        bincode::deserialize(&[4, 0, 0, 0, 0, 0, 0, 0, 97, 105, 117, 101]).unwrap();
    let de_rgb: Box<dyn Color> = bincode::deserialize(&ser_rgb).unwrap();
    println!("{:?}", de_rgb);

    let opt = bincode::DefaultOptions::new();

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Unchi {
        r: Box<u8>,
        g: Box<u8>,
        b: Box<u8>,
    }
    let unchi_ser = bincode::serialize(&Unchi {
        r: Box::new(0),
        g: Box::new(0),
        b: Box::new(0),
    })
    .unwrap();
    println!("{:?}", unchi_ser);
    let unchi_de: Unchi = bincode::deserialize(&unchi_ser).unwrap();
    println!("{:?}", unchi_de);

    let bytes = Bytes::new(&rgb_bytes);
    println!("{:?}", bytes);
    //let (de_tp_expectation, tp_bytes) = common::tp_setup();

    todo!();
}
