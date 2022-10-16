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
                    unsafe { bytes.as_ptr().cast::<T>().read() }
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

macro_rules! impl_scannable_for_float {
    ( $( $type:ty : $type_size:expr ; $int_type:ty ),+ ) => {
        $(
            impl Scannable<$type_size> for $type {

                fn from_bytes<T: Scannable<$type_size>>(bytes: [u8; $type_size]) -> T {
                    // SAFETY: size of input and output is always the same
                    unsafe { bytes.as_ptr().cast::<T>().read() }
                }

                fn to_bytes(self) -> [u8; $type_size] {
                    self.to_ne_bytes()
                }

                 fn eq(&self, bytes: [u8; $type_size]) -> bool {
                    /// Let's visualize this mask with a f16.
                    /// This type has 16 bits, 1 for sign, 5 for exponent, and 10 for the mantissa:
                    /// S EEEEE MMMMMMMMMM

                    // If we substitute the constant with the numeric value and operate:
                    /// !((1 << (10 / 2)) - 1)
                    /// !((1 << 5) - 1)
                    /// !(0b00000000_00100000 - 1)
                    /// !(0b00000000_00011111)
                    /// 0b11111111_11100000

                    /// So effectively, half of the mantisssa bit will be masked to 0.
                    /// For the f16 example, this makes us lose 5 bits of precision.
                    /// Comparing two floating point values with their last five bits
                    /// truncated is equivalent to checking if they are "roughly equal"!
                    const MASK: $int_type = !((1 << (<$type>::MANTISSA_DIGITS / 2)) - 1);

                    let other = <$type>::from_ne_bytes(bytes);

                    let this = <$type>::from_bits(self.to_bits() & MASK);
                    let other = <$type>::from_bits(other.to_bits() & MASK);

                    this == other
                }

                 fn cmp(&self, bytes: [u8; $type_size]) -> Ordering {
                    let other = <$type>::from_ne_bytes(bytes);
                    self.total_cmp(&other)
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
impl_scannable_for_float!(f32: 4; u32, f64: 8; u64);

#[cfg(test)]
mod scannable_tests {
    use super::*;

    #[test]
    fn f32_roughly_eq() {
        let left = 0.25f32;
        let right = 0.25000123f32;
        assert_ne!(left, right);
        let right = right.to_bytes();
        assert!(Scannable::eq(&left, right));
    }
}
