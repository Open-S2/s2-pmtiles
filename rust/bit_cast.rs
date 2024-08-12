use core::mem::transmute;

/// All encoding and decoding is done via u64.
/// So all types must implement this trait to be able to be encoded and decoded.
pub trait BitCast: Sized {
    /// Convert the value to a u64.
    fn to_u64(&self) -> u64;
    /// Convert a u64 to the value.
    fn from_u64(value: u64) -> Self;
}
impl BitCast for u64 {
    fn to_u64(&self) -> u64 {
        *self
    }
    fn from_u64(value: u64) -> Self {
        value
    }
}
impl BitCast for i64 {
    fn to_u64(&self) -> u64 {
        unsafe { transmute::<i64, u64>(*self) }
    }
    fn from_u64(value: u64) -> Self {
        unsafe { transmute::<u64, i64>(value) }
    }
}
impl BitCast for f64 {
    fn to_u64(&self) -> u64 {
        (*self).to_bits()
    }
    fn from_u64(value: u64) -> Self {
        f64::from_bits(value)
    }
}
impl BitCast for u32 {
    fn to_u64(&self) -> u64 {
        *self as u64
    }
    fn from_u64(value: u64) -> Self {
        value as u32
    }
}
impl BitCast for i32 {
    fn to_u64(&self) -> u64 {
        unsafe { transmute::<i32, u32>(*self) as u64 }
    }
    fn from_u64(value: u64) -> Self {
        unsafe { transmute::<u32, i32>(value as u32) }
    }
}
impl BitCast for f32 {
    fn to_u64(&self) -> u64 {
        (*self).to_bits() as u64
    }
    fn from_u64(value: u64) -> Self {
        f32::from_bits(value as u32)
    }
}
impl BitCast for u16 {
    fn to_u64(&self) -> u64 {
        *self as u64
    }
    fn from_u64(value: u64) -> Self {
        value as u16
    }
}
impl BitCast for i16 {
    fn to_u64(&self) -> u64 {
        unsafe { transmute::<i16, u16>(*self) as u64 }
    }
    fn from_u64(value: u64) -> Self {
        unsafe { transmute::<u16, i16>(value as u16) }
    }
}
impl BitCast for u8 {
    fn to_u64(&self) -> u64 {
        *self as u64
    }
    fn from_u64(value: u64) -> Self {
        value as u8
    }
}
impl BitCast for i8 {
    fn to_u64(&self) -> u64 {
        unsafe { transmute::<i8, u8>(*self) as u64 }
    }
    fn from_u64(value: u64) -> Self {
        unsafe { transmute::<u8, i8>(value as u8) }
    }
}
impl BitCast for bool {
    fn to_u64(&self) -> u64 {
        if *self {
            1
        } else {
            0
        }
    }
    fn from_u64(value: u64) -> Self {
        value != 0
    }
}
impl BitCast for usize {
    fn to_u64(&self) -> u64 {
        *self as u64
    }
    fn from_u64(value: u64) -> Self {
        value as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitcast_u64() {
        // from
        assert_eq!(u64::from_u64(0), 0);
        assert_eq!(u64::from_u64(1), 1);
        assert_eq!(u64::from_u64(0xffffffff), 0xffffffff);

        // to
        assert_eq!(u64::to_u64(&0), 0);
        assert_eq!(u64::to_u64(&1), 1);
        assert_eq!(u64::to_u64(&0xffffffff), 0xffffffff);
    }

    #[test]
    fn test_bitcast_i64() {
        // from
        assert_eq!(i64::from_u64(0), 0);
        assert_eq!(i64::from_u64(1), 1);
        assert_eq!(i64::from_u64(18446744073709551615), -1);

        // to
        assert_eq!(i64::to_u64(&0), 0);
        assert_eq!(i64::to_u64(&1), 1);
        assert_eq!(i64::to_u64(&-1), 18446744073709551615);
    }

    #[test]
    fn test_bitcast_f64() {
        // from
        assert_eq!(f64::from_u64(0), 0.0);
        assert_eq!(f64::from_u64(4607182418800017408), 1.0);
        assert_eq!(f64::from_u64(13830554455654793216), -1.0);

        // to
        assert_eq!(f64::to_u64(&0.0), 0);
        assert_eq!(f64::to_u64(&1.0), 4607182418800017408);
        assert_eq!(f64::to_u64(&-1.0), 13830554455654793216);
    }

    #[test]
    fn test_bitcast_u32() {
        // from
        assert_eq!(u32::from_u64(0), 0);
        assert_eq!(u32::from_u64(1), 1);
        assert_eq!(u32::from_u64(0xffffffff), 0xffffffff);

        // to
        assert_eq!(u32::to_u64(&0), 0);
        assert_eq!(u32::to_u64(&1), 1);
        assert_eq!(u32::to_u64(&0xffffffff), 0xffffffff);
    }

    #[test]
    fn test_bitcast_i32() {
        // from
        assert_eq!(i32::from_u64(0), 0);
        assert_eq!(i32::from_u64(1), 1);
        assert_eq!(i32::from_u64(4294967295), -1);

        // to
        assert_eq!(i32::to_u64(&0), 0);
        assert_eq!(i32::to_u64(&1), 1);
        assert_eq!(i32::to_u64(&-1), 4294967295);
    }

    #[test]
    fn test_bitcast_f32() {
        // from
        assert_eq!(f32::from_u64(0), 0.0);
        assert_eq!(f32::from_u64(1065353216), 1.0);
        assert_eq!(f32::from_u64(3212836864), -1.0);

        // to
        assert_eq!(f32::to_u64(&0.0), 0);
        assert_eq!(f32::to_u64(&1.0), 1065353216);
        assert_eq!(f32::to_u64(&-1.0), 3212836864);
    }

    #[test]
    fn test_bitcast_u16() {
        // from
        assert_eq!(u16::from_u64(0), 0);
        assert_eq!(u16::from_u64(1), 1);
        assert_eq!(u16::from_u64(0xffff), 0xffff);

        // to
        assert_eq!(u16::to_u64(&0), 0);
        assert_eq!(u16::to_u64(&1), 1);
        assert_eq!(u16::to_u64(&0xffff), 0xffff);
    }

    #[test]
    fn test_bitcast_i16() {
        // from
        assert_eq!(i16::from_u64(0), 0);
        assert_eq!(i16::from_u64(1), 1);
        assert_eq!(i16::from_u64(65535), -1);

        // to
        assert_eq!(i16::to_u64(&0), 0);
        assert_eq!(i16::to_u64(&1), 1);
        assert_eq!(i16::to_u64(&-1), 65535);
    }

    #[test]
    fn test_bitcast_u8() {
        // from
        assert_eq!(u8::from_u64(0), 0);
        assert_eq!(u8::from_u64(1), 1);
        assert_eq!(u8::from_u64(255), 255);

        // to
        assert_eq!(u8::to_u64(&0), 0);
        assert_eq!(u8::to_u64(&1), 1);
        assert_eq!(u8::to_u64(&255), 255);
    }

    #[test]
    fn test_bitcast_i8() {
        // from
        assert_eq!(i8::from_u64(0), 0);
        assert_eq!(i8::from_u64(1), 1);
        assert_eq!(i8::from_u64(255), -1);

        // to
        assert_eq!(i8::to_u64(&0), 0);
        assert_eq!(i8::to_u64(&1), 1);
        assert_eq!(i8::to_u64(&-1), 255);
    }

    #[test]
    fn test_bitcast_bool() {
        // from
        assert!(!bool::from_u64(0));
        assert!(bool::from_u64(1));
        assert!(bool::from_u64(2));

        // to
        assert_eq!(bool::to_u64(&false), 0);
        assert_eq!(bool::to_u64(&true), 1);
    }

    #[test]
    fn test_bitcast_usize() {
        // from
        assert_eq!(usize::from_u64(0), 0);
        assert_eq!(usize::from_u64(1), 1);
        assert_eq!(usize::from_u64(4294967295), 4294967295);

        // to
        assert_eq!(usize::to_u64(&0), 0);
        assert_eq!(usize::to_u64(&1), 1);
        assert_eq!(usize::to_u64(&4294967295), 4294967295);
    }
}
