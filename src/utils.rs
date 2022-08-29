//! Utils
//!
//!
//!

use bytes::{Bytes, BytesMut};

pub trait AsBytes {
    fn as_bytes(&self) -> Bytes;
}

pub trait ExtendBytesMut {
    fn extend_bytes(&self, extended: &mut BytesMut);
}
