use std::{fs, path::Path};

use mers_libs::file::File;
use mers_libs::{parse, VDataEnum};

#[test]
fn run_all() {
    for file in fs::read_dir(Path::new(file!()).parent().unwrap())
        .unwrap()
        .filter_map(|v| v.ok())
    {
        if let Some(file_name) = file.file_name().to_str() {
            if file_name.ends_with(".mers") {
                eprintln!("Checking {}", file_name);
                let mut file = File::new(fs::read_to_string(file.path()).unwrap(), file.path());
                // has to return true, otherwise the test will fail
                assert!(matches!(
                    parse::parse(&mut file).unwrap().run(vec![]).data().0,
                    VDataEnum::Bool(true)
                ));
            }
        }
    }
}
