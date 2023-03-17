use std::time::Instant;

pub(crate) mod parse;
pub(crate) mod script;

fn main() {
    let script = parse::parse::parse(&mut parse::file::File::new(
        std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap(),
    ))
    .unwrap();
    println!(" - - - - -");
    let start = Instant::now();
    let out = script.run(std::env::args().collect());
    let elapsed = start.elapsed();
    println!(" - - - - -");
    println!("Output ({}s)\n{out}", elapsed.as_secs_f64());
}
