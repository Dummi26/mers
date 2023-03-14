use std::time::Instant;

use crate::script::value::{VSingleType, VType};

pub(crate) mod parse;
pub(crate) mod script;

fn main() {
    let val: VType = VSingleType::Tuple(vec![
        VSingleType::Int.into(),
        VSingleType::String.into(),
        VSingleType::String.into(),
    ])
    .into();
    let case: VType = VSingleType::Tuple(vec![
        VType {
            types: vec![VSingleType::Tuple(vec![]).into(), VSingleType::Int.into()],
        },
        VSingleType::String.into(),
        VSingleType::String.into(),
    ])
    .into();
    assert!(val.fits_in(&case).is_empty());
    let script = parse::parse::parse(&mut parse::file::File::new(
        std::fs::(std::env::args().nth(1).unwrap()).unwrap(),
    ))
    .unwrap();
    println!(" - - - - -");
    let start = Instant::now();
    let out = script.run(std::env::args().collect());
    let elapsed = start.elapsed();
    println!(" - - - - -");
    println!("Output ({}s)\n{out}", elapsed.as_secs_f64());
}
