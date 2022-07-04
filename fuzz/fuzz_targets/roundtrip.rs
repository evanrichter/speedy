#![no_main]
use std::borrow::Cow;

use libfuzzer_sys::fuzz_target;

use speedy::{Endianness, Readable, Writable};

#[derive(Debug, Readable, Writable, PartialEq)]
enum PlainEnum {
    A,
    B,
    C,
    D,
}

#[derive(Debug, Readable, Writable, PartialEq)]
enum Enum {
    A(u8),
    B(()),
    C(Vec<PlainEnum>),
    D(i128),
}

#[derive(Debug, Readable, Writable, PartialEq)]
enum FloatEnum {
    A(Enum),
    E(Option<f32>),
}

#[derive(PartialEq, Debug, Readable, Writable)]
struct Struct<'a> {
    number: u64,
    string: String,
    vector: Vec<u8>,
    cow: Cow<'a, [i64]>,
    enumeration: Enum,
    tuple: (u128, i8, (), PlainEnum, String),
}

#[derive(Debug, PartialEq, Readable, Writable)]
struct Struct2 {
    byte_count: u8,
    #[speedy(length = byte_count / 4)]
    data: Vec<u32>,
}

#[derive(Debug, Readable, Writable, PartialEq)]
struct FloatStruct<'a> {
    a: Struct<'a>,
    b: f64,
}

macro_rules! round_trip {
    ($ty:ty, $data:ident, $equality:expr, $ctx:expr) => {{
        #[cfg(feature = "debug")]
        println!("roundtripping {} ({})", stringify!($ty), stringify!($ctx));

        let x: Result<$ty, _> = <$ty as Readable<_>>::read_from_buffer_with_ctx($ctx, $data);
        if let Ok(inner) = x {
            #[cfg(feature = "debug")]
            dbg!(&inner);

            let ser = <$ty as Writable<_>>::write_to_vec_with_ctx(&inner, $ctx)
                .expect("a deserialized type should serialize");
            #[cfg(feature = "debug")]
            dbg!(&ser);

            let des: $ty = <$ty as Readable<_>>::read_from_buffer_with_ctx($ctx, &ser)
                .expect("a serialized type should deserialize");
            #[cfg(feature = "debug")]
            dbg!(&des);

            if $equality {
                assert_eq!(inner, des, "roundtripped object changed");
            }
        }
    }};
}

macro_rules! from_bytes {
    ($ty:ty, $data:ident, $equality:expr) => {{
        round_trip!($ty, $data, $equality, Endianness::LittleEndian);
        round_trip!($ty, $data, $equality, Endianness::BigEndian);
        round_trip!(Option<$ty>, $data, $equality, Endianness::LittleEndian);
        round_trip!(Option<$ty>, $data, $equality, Endianness::BigEndian);
        round_trip!(Vec<$ty>, $data, $equality, Endianness::LittleEndian);
        round_trip!(Vec<$ty>, $data, $equality, Endianness::BigEndian);
    }};
}

fuzz_target!(|data: &[u8]| {
    #[cfg(feature = "debug")]
    println!("bytes: {:#02x?}", data);

    from_bytes!(bool, data, true);
    from_bytes!(i8, data, true);
    from_bytes!(i16, data, true);
    from_bytes!(i32, data, true);
    from_bytes!(i64, data, true);
    from_bytes!(i128, data, true);
    from_bytes!(u8, data, true);
    from_bytes!(u16, data, true);
    from_bytes!(u32, data, true);
    from_bytes!(u64, data, true);
    from_bytes!(u128, data, true);
    from_bytes!(f32, data, false);
    from_bytes!(f64, data, false);
    from_bytes!(char, data, true);
    from_bytes!(&str, data, true);
    from_bytes!((), data, true);
    from_bytes!(PlainEnum, data, true);
    from_bytes!(Enum, data, true);
    from_bytes!(FloatEnum, data, false);
    from_bytes!(Struct, data, true);
    from_bytes!(Struct2, data, true);
    from_bytes!(FloatStruct, data, false);
});
