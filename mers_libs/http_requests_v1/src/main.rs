use mers_libs::prelude::*;

fn main() {
    let mut my_lib = MyLib::new(
        "http".to_string(), // "HTTP requests for MERS".to_string(),
        (0, 0),
        "desc".to_string(), // "basic HTTP functionality for mers. warning: this is fully single-threaded.".to_string(),
        vec![
            // http_get
            (
                "http_get".to_string(),
                vec![
                    // (String) -> String/Err(ErrBuildingRequest(String)/ErrGettingResponseText(String))
                    (
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
                    ),
                ],
            ),
        ],
    );
    let err_general = my_lib
        .get_enums()
        .iter()
        .find(|(name, _)| name.as_str() == "Err")
        .unwrap()
        .1;
    let err_building_request = my_lib
        .get_enums()
        .iter()
        .find(|(name, _)| name.as_str() == "ErrBuildingRequest")
        .unwrap()
        .1;
    let err_getting_response_text = my_lib
        .get_enums()
        .iter()
        .find(|(name, _)| name.as_str() == "ErrGettingResponseText")
        .unwrap()
        .1;
    my_lib.callbacks.run_function.consuming = Some(Box::new(move |msg| {
        if let VDataEnum::String(url) = &msg.msg.args[0].data {
            let url = url.clone();
            std::thread::spawn(move || {
                let r = match reqwest::blocking::get(url) {
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
                };
                msg.respond(r)
            });
        } else {
            unreachable!()
        }
    }));
    // because we handle all callbacks, this never returns Err(unhandeled message).
    // it returns Ok(()) if mers exits (i/o error in stdin/stdout), so we also exit if that happens.
    my_lib.get_next_unhandled_message().unwrap();
}
// fn run_function(f: ()) {
//     let return_value = match f.function {
//         0 => {
//             // http_get
//             if let VDataEnum::String(url) = &f.args[0].data {
//                 match reqwest::blocking::get(url) {
//                     Ok(response) => match response.text() {
//                         Ok(text) => VDataEnum::String(text).to(),
//                         Err(e) => VDataEnum::EnumVariant(
//                             err_general,
//                             Box::new(
//                                 VDataEnum::EnumVariant(
//                                     err_getting_response_text,
//                                     Box::new(VDataEnum::String(e.to_string()).to()),
//                                 )
//                                 .to(),
//                             ),
//                         )
//                         .to(),
//                     },
//                     Err(e) => VDataEnum::EnumVariant(
//                         err_general,
//                         Box::new(
//                             VDataEnum::EnumVariant(
//                                 err_building_request,
//                                 Box::new(VDataEnum::String(e.to_string()).to()),
//                             )
//                             .to(),
//                         ),
//                     )
//                     .to(),
//                 }
//             } else {
//                 unreachable!()
//             }
//         }
//         _ => unreachable!(),
//     };
//     f.done(&mut stdout, return_value)
// }
