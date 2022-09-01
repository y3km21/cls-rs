//! Utils
//!
//!
//!

use bytes::BytesMut;
use js_sys::Number;
use num_traits::{Num, NumCast};

pub trait ExtendBytesMut {
    /// Append to given BytesMut.
    fn extend_bytes(&self, extended: &mut BytesMut);
}

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

#[cfg(feature = "web")]
pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Cast JS Number to Primitive num type
pub fn cast_js_number<T: Num + NumCast>(js_number: Number) -> Option<T> {
    js_number
        .as_f64()
        .map(|float_64| T::from(float_64))
        .flatten()
}
