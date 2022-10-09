use std::cmp::Ordering;

// TODO: cleanup docs
pub trait Scannable<const SIZE: usize>: Copy {
    // TODO: write document
    fn from_bytes<T: Scannable<SIZE>>(bytes: [u8; SIZE]) -> T;
    /// Returns `true` if the current instance is considered equal to the given chunk of memory.
    ///
    /// Callers must `assert_eq!(left.len(), right.len())`, and the length must also match that of
    /// the length represented by `Self`.
    fn eq(&self, bytes: [u8; SIZE]) -> bool;

    /// Compares `self` to the given chunk of memory.
    ///
    /// Callers must `assert_eq!(left.len(), right.len())`, and the length must also match that of
    /// the length represented by `Self`.
    fn cmp(&self, bytes: [u8; SIZE]) -> Ordering;

    /// Substracts the given chunk of memory from `self`.
    ///
    /// Callers must `assert_eq!(left.len(), right.len())`, and the length must also match that of
    /// the length represented by `Self`.
    fn sub(&mut self, bytes: [u8; SIZE]);

    /// Substracts `self` from the given chunk of memory.
    ///
    /// Callers must `assert_eq!(left.len(), right.len())`, and the length must also match that of
    /// the length represented by `Self`.
    fn rsub(&mut self, bytes: [u8; SIZE]);
}

macro_rules! impl_scannable_for_int {
    ( $( $type:ty : $type_size:expr ),+ ) => {
        $(
            // SAFETY: caller is responsible to `assert_eq!(memory.len(), mem::size_of::<T>())`
            impl Scannable<$type_size> for $type {
                // type Type = $type;

                fn from_bytes<T: Scannable<$type_size>>(bytes: [u8; $type_size]) -> T {
                    unsafe { bytes.as_ptr().cast::<T>().read_unaligned() }
                }

                 fn eq(&self, bytes: [u8; $type_size]) -> bool {
                    let other = <$type>::from_ne_bytes(bytes);
                    *self == other
                }

                 fn cmp(&self, bytes: [u8; $type_size]) -> Ordering {
                    let other = <$type>::from_ne_bytes(bytes);
                    <$type as Ord>::cmp(self, &other)
                }

                fn sub(&mut self, bytes: [u8; $type_size]){
                    todo!()
                }

                fn rsub(&mut self, bytes: [u8; $type_size]){
                    todo!()
                }
            }
        )*
    };
}

impl_scannable_for_int!(i8: 1, u8: 1, i16: 2, u16:2 , i32: 4, u32: 4, i64: 8, u64: 8);
