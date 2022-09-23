//! Common

use bytes::BytesMut;
use nom;
/// ExtendBytesMut Trait
///
///
pub trait ExtendBytesMut {
    /// Append to given BytesMut.
    fn extend_bytes(&self, extended: &mut BytesMut);
}

/// TryFromBytes
///
pub trait TryFromBytes {
    /// Input bytes try into Self.
    ///
    /// Using nom
    fn try_from_bytes(input: &[u8]) -> nom::IResult<&[u8], Self>
    where
        Self: Sized;
}

/// ClsSize Trait
pub trait ClsSize {
    /// Returns the byte size in the cls file, not including the size header.
    fn size_contents_in_cls(&self) -> u32;

    /// Returns the byte size in the cls file, including the size header.
    /// # Note
    /// If not overridden, it is a same [`Self::size_contents_in_cls`]
    fn size_in_cls(&self) -> u32 {
        self.size_contents_in_cls()
    }
}
