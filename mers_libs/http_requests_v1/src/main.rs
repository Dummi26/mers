use mers::{
    libs::inlib::{MyLib, MyLibTask},
    script::{
        val_data::VDataEnum,
        val_type::{VSingleType, VType},
    },
};

fn main() {
    let (mut my_lib, mut run) = MyLib::new(
        "HTTP requests for MERS".to_string(),
        (0, 0),
        "basic HTTP functionality for mers. warning: this is fully single-threaded.".to_string(),
        vec![(
            "http_get".to_string(),
            vec![VSingleType::String.to()],
            VType {
                types: vec![
                    VSingleType::String,
                    VSingleType::EnumVariantS(
                        format!("Err"),
                        VType {
                            types: vec![
                                VSingleType::EnumVariantS(
                                    format!("ErrBuildingRequest"),
                                    VSingleType::String.to(),
                                ),
                                VSingleType::EnumVariantS(
                                    format!("ErrGettingResponseText"),
                                    VSingleType::String.to(),
                                ),
                            ],
                        },
                    ),
                ],
            },
        )],
    );
    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    let mut err_general = 0;
    let mut err_building_request = 0;
    let mut err_getting_response_text = 0;
    loop {
        run = match my_lib.run(run, &mut stdin, &mut stdout) {
            MyLibTask::None(v) => v,
            MyLibTask::FinishedInit(v) => {
                err_general = my_lib.get_enum("Err").unwrap();
                err_building_request = my_lib.get_enum("ErrBuildingRequest").unwrap();
                err_getting_response_text = my_lib.get_enum("ErrGettingResponseText").unwrap();
                v
            }
            MyLibTask::RunFunction(f) => {
                let return_value = match f.function {
                    0 => {
                        // http_get
                        if let VDataEnum::String(url) = &f.args[0].data {
                            match reqwest::blocking::get(url) {
                                Ok(response) => match response.text() {
                                    Ok(text) => VDataEnum::String(text).to(),
                                    Err(e) => VDataEnum::EnumVariant(
                                        err_general,
                                        Box::new(
                                            VDataEnum::EnumVariant(
                                                err_getting_response_text,
                                                Box::new(VDataEnum::String(e.to_string()).to()),
                                            )
                                            .to(),
                                        ),
                                    )
                                    .to(),
                                },
                                Err(e) => VDataEnum::EnumVariant(
                                    err_general,
                                    Box::new(
                                        VDataEnum::EnumVariant(
                                            err_building_request,
                                            Box::new(VDataEnum::String(e.to_string()).to()),
                                        )
                                        .to(),
                                    ),
                                )
                                .to(),
                            }
                        } else {
                            unreachable!()
                        }
                    }
                    _ => unreachable!(),
                };
                f.done(&mut stdout, return_value)
            }
        }
    }
}
