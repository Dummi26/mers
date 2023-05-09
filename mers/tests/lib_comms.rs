use std::io::Cursor;

use mers_libs::prelude::*;
use mers_libs::{ByteData, ByteDataA};

#[test]
fn list_type() {
    let a: Vec<i32> = vec![14, 26];
    let bytes = a.as_byte_data_vec();
    println!("{bytes:?}");
    assert_eq!(
        Vec::<i32>::from_byte_data(&mut Cursor::new(bytes)).unwrap(),
        a
    );

    let a = VSingleType::List(VSingleType::Int.to()).to();
    assert_eq!(
        VType::from_byte_data(&mut Cursor::new(a.as_byte_data_vec())).unwrap(),
        a
    );

    let a = VSingleType::Tuple(vec![
        VType {
            types: vec![VSingleType::Tuple(vec![]), VSingleType::Int],
        },
        VSingleType::String.to(),
        VSingleType::EnumVariant(12, VSingleType::Float.to()).to(),
    ])
    .to();
    assert_eq!(
        VType::from_byte_data(&mut Cursor::new(a.as_byte_data_vec())).unwrap(),
        a
    );
}
