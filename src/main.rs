use std::time::Instant;

use crate::script::value::{VSingleType, VType};

pub(crate) mod parse;
pub(crate) mod script;

fn main() {
    let str1: VType = VSingleType::String.into();
    assert!(str1.fits_in(&VSingleType::String.into()).is_empty());
    let script = parse::parse::parse(&mut parse::file::File::new(
        std::fs::read_to_string("/tmp/script.txt").unwrap(),
    ))
    .unwrap();
    println!(" - - - - -");
    let start = Instant::now();
    let out = script.run(std::env::args().collect());
    let elapsed = start.elapsed();
    println!(" - - - - -");
    println!("Output ({}s)\n{out:?}", elapsed.as_secs_f64());
}
