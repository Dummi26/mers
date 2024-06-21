mod tasks;

use colored::Colorize;

use std::{fs, path::PathBuf, time::Duration};

use clap::Parser;
use notify::Watcher;

use mers_lib::prelude_compile::*;

#[derive(Parser)]
pub(crate) struct Args {
    file: PathBuf,
    /// Skip this many tasks, i.e. start at task #<skip>, where 0 is `Hello, World`.
    #[arg(long, default_value_t = 0)]
    skip: usize,
}

fn main() {
    let args = Args::parse();
    let (s, r) = std::sync::mpsc::channel();
    let mut file_watcher = notify::recommended_watcher(move |e: Result<notify::Event, _>| {
        let e = e.unwrap();
        match e.kind {
            notify::EventKind::Modify(k) => match k {
                notify::event::ModifyKind::Data(_) => {
                    s.send(()).unwrap();
                }
                _ => {}
            },
            _ => {}
        }
    })
    .unwrap();
    fs::write(&args.file, "\"Hello, World!\".println\n").unwrap();
    file_watcher
        .watch(&args.file, notify::RecursiveMode::NonRecursive)
        .unwrap();
    for (task_i, (init_msg, cfg, mut task)) in
        tasks::tasks(&args).into_iter().enumerate().skip(args.skip)
    {
        eprintln!("\n\n\n\n\n{}", format!(" -= {task_i} =-").bright_black());
        eprintln!("{}", init_msg);
        loop {
            // wait for file change
            r.recv().unwrap();
            std::thread::sleep(Duration::from_millis(50));
            _ = r.try_iter().count();
            let code = fs::read_to_string(&args.file).unwrap();
            let mut src = Source::new(code);
            let v = parse(&mut src).and_then(|v| {
                let config = cfg();
                let (mut i1, i2, mut i3) = config.infos();
                v.compile(&mut i1, CompInfo::default())
                    .and_then(|v| v.check(&mut i3, None).map(|t| (t, v, i1, i2, i3)))
            });
            eprintln!("\n{}\n", "- - - - - - - - - -".bright_black());
            if task(v, src) {
                break;
            }
        }
    }
    eprintln!("\nThere are no more tasks here. Feel free to suggest new tasks or changes to existing ones, and thanks for taking the time to check out mers :)");
}
