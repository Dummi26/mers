use std::time::Instant;

pub mod libs;
pub mod parse;
pub mod script;

fn main() {
    let path = std::env::args().nth(1).unwrap();
    let script = parse::parse::parse(&mut parse::file::File::new(
        std::fs::read_to_string(&path).unwrap(),
        path.into(),
    ))
    .unwrap();
    println!(" - - - - -");
    let start = Instant::now();
    let out = script.run(std::env::args().skip(2).collect());
    let elapsed = start.elapsed();
    println!(" - - - - -");
    println!("Output ({}s)\n{out}", elapsed.as_secs_f64());
}
