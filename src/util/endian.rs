macro_rules! define_slice_to_be {
    ($name:ident, $type: ty) => {
        #[inline]
        pub fn $name(slice: &[u8]) -> $type {
            assert_eq!(slice.len(), ::std::mem::size_of::<$type>());
            let mut res = 0;
            for i in 0..::std::mem::size_of::<$type>() {
                res |= (slice[i] as $type) << (::std::mem::size_of::<$type>() - i - 1) * 8;
            }
            res
        }
    };
}
macro_rules! define_slice_to_le {
    ($name:ident, $type: ty) => {
        #[inline]
        pub fn $name(slice: &[u8]) -> $type {
            assert_eq!(slice.len(), ::std::mem::size_of::<$type>());
            let mut res = 0;
            for i in 0..::std::mem::size_of::<$type>() {
                res |= (slice[i] as $type) << i * 8;
            }
            res
        }
    };
}
macro_rules! define_be_to_array {
    ($name: ident, $type: ty, $byte_len: expr) => {
        #[inline]
        pub fn $name(val: $type) -> [u8; $byte_len] {
            assert_eq!(::std::mem::size_of::<$type>(), $byte_len);
            let mut res = [0; $byte_len];
            for i in 0..$byte_len {
                res[i] = ((val >> ($byte_len - i - 1) * 8) & 0xff) as u8;
            }
            res
        }
    };
}

define_slice_to_be!(slice_to_u32_be, u32);
define_be_to_array!(u32_to_array_be, u32, 4);
define_slice_to_le!(slice_to_u16_le, u16);
define_slice_to_le!(slice_to_u32_le, u32);
define_slice_to_le!(slice_to_u64_le, u64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endianness_test() {
        assert_eq!(slice_to_u32_be(&[0xde, 0xad, 0xbe, 0xef]), 0xdeadbeef);
        assert_eq!(u32_to_array_be(0xdeadbeef), [0xde, 0xad, 0xbe, 0xef]);
        assert_eq!(slice_to_u16_le(&[0xad, 0xde]), 0xdead);
        assert_eq!(slice_to_u32_le(&[0xef, 0xbe, 0xad, 0xde]), 0xdeadbeef);
        assert_eq!(
            slice_to_u64_le(&[0xef, 0xbe, 0xad, 0xde, 0xfe, 0xca, 0xad, 0x1b]),
            0x1badcafedeadbeef
        );
    }
}
