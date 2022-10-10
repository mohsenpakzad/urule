use std::cmp::Ordering;

/// A scannable type representor.
///
/// The trait functions determine a scannable type behaviors.
pub trait Scannable<const SIZE: usize>: Copy {
    /// Create a Scannable value from its memory representation as a byte array.
    fn from_bytes<T: Scannable<SIZE>>(bytes: [u8; SIZE]) -> T;

    /// Return the memory representation of this Scannable as a byte array.
    fn to_bytes(self) -> [u8; SIZE];

    /// Returns `true` if the current instance is considered equal to the given chunk of memory.
    fn eq(&self, bytes: [u8; SIZE]) -> bool;

    /// Compares `self` to the given chunk of memory.
    fn cmp(&self, bytes: [u8; SIZE]) -> Ordering;

    /// Return subtract value from `self` and given chunk of memory.
    fn sub(&self, other: [u8; SIZE]) -> [u8; SIZE];
}

macro_rules! impl_scannable_for_int {
    ( $( $type:ty : $type_size:expr ),+ ) => {
        $(
            impl Scannable<$type_size> for $type {

                fn from_bytes<T: Scannable<$type_size>>(bytes: [u8; $type_size]) -> T {
                    // SAFETY: size of input and output is always the same
                    unsafe { bytes.as_ptr().cast::<T>().read_unaligned() }
                }

                fn to_bytes(self) -> [u8; $type_size] {
                    self.to_ne_bytes()
                }

                 fn eq(&self, bytes: [u8; $type_size]) -> bool {
                    let other = <$type>::from_ne_bytes(bytes);
                    *self == other
                }

                 fn cmp(&self, bytes: [u8; $type_size]) -> Ordering {
                    let other = <$type>::from_ne_bytes(bytes);
                    <$type as Ord>::cmp(self, &other)
                }

                fn sub(&self, bytes: [u8; $type_size]) -> [u8; $type_size] {
                    let other = <$type>::from_ne_bytes(bytes);
                    (*self - other).to_ne_bytes()
                }
            }
        )+
    };
}

impl_scannable_for_int!(i8: 1, u8: 1, i16: 2, u16:2 , i32: 4, u32: 4, i64: 8, u64: 8);
