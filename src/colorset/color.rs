//! Color Segment
//!
use serde::{de::Visitor, ser::SerializeStruct, Deserialize, Serialize};

type ColorName = String;

pub struct ColorSegment {
    color: Box<dyn Color>,
    color_name: Option<ColorName>,
}

#[derive(Debug)]
pub struct RGB {
    red: u8,
    green: u8,
    blue: u8,
}

impl RGB {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}

impl Color for RGB {}

impl Serialize for RGB {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Color", 4)?;
        s.serialize_field("red", &self.red)?;
        s.serialize_field("green", &self.green)?;
        s.serialize_field("blue", &self.blue)?;
        s.serialize_field("transparency", &0xFFu8)?;
        s.end()
    }
}

#[derive(Debug)]
pub struct TransparentColor;

impl TransparentColor {
    pub fn new() -> Self {
        TransparentColor
    }
}

impl Color for TransparentColor {}

impl Serialize for TransparentColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Color", 4)?;
        s.serialize_field("red", &0u8)?;
        s.serialize_field("green", &0u8)?;
        s.serialize_field("blue", &0u8)?;
        s.serialize_field("transparency", &0u8)?;
        s.end()
    }
}

pub trait Color: std::fmt::Debug {}

struct ColorVisitor;

impl<'de> Visitor<'de> for ColorVisitor {
    type Value = Box<dyn Color>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "Colors")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        println!("in visit_bytes: {:?}", v);
        let transparency = v.get(3).ok_or(E::missing_field("transparency"))?;

        if *transparency == 0 {
            return Ok(Box::new(TransparentColor::new()));
        }

        let red = v.get(0).unwrap();
        let green = v.get(1).unwrap();
        let blue = v.get(2).unwrap();

        Ok(Box::new(RGB::new(*red, *green, *blue)))
    }
}

impl<'de> Deserialize<'de> for Box<dyn Color> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(ColorVisitor)
    }
}
