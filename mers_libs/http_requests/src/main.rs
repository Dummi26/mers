fn main() {
    let (mut my_lib, mut run) = MyLib::new(
        "GUI-Iced".to_string(),
        (0, 0),
        "A basic GUI library for mers.".to_string(),
        vec![(
            "http_get".to_string(),
            vec![VSingleType::String],
            VType {
                types: vec![VSingleType::Tuple(vec![]), VSingleType::String],
            },
        )],
    );
    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    let mut layout = Layout::Row(vec![]);
    loop {
        run = match my_lib.run(run, &mut stdin, &mut stdout) {
            MyLibTask::None(v) => v,
            MyLibTask::RunFunction(mut f) => {
                let return_value = match f.function {
                    0 => VDataEnum::List(VSingleType::Int.to(), vec![]).to(),
                    _ => unreachable!(),
                };
                f.done(&mut stdout, return_value)
            }
        }
    }
}
