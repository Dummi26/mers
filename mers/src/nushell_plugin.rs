use std::{fs, path::PathBuf};

use nu_plugin::{serve_plugin, MsgPackSerializer, Plugin};
use nu_protocol::{PluginExample, PluginSignature, ShellError, Span, Spanned, SyntaxShape, Value};

use crate::{
    lang::{
        global_info::GlobalScriptInfo,
        val_data::{VData, VDataEnum},
    },
    parsing,
};

pub fn main() {
    serve_plugin(&mut MersNuPlugin(), MsgPackSerializer {});
}

struct MersNuPlugin();

impl Plugin for MersNuPlugin {
    fn signature(&self) -> Vec<nu_protocol::PluginSignature> {
        vec![PluginSignature::build("mers-nu")
            .required(
                "mers",
                SyntaxShape::String,
                "the path to the .mers file to run or the mers source code if -e is set",
            )
            .optional(
                "args",
                SyntaxShape::List(Box::new(SyntaxShape::String)),
                "the arguments passed to the mers program. defaults to an empty list.",
            )
            .switch(
                "execute",
                "instead of reading from a file, interpret the 'mers' input as source code",
                Some('e'),
            )]
    }
    fn run(
        &mut self,
        name: &str,
        call: &nu_plugin::EvaluatedCall,
        input: &nu_protocol::Value,
    ) -> Result<nu_protocol::Value, nu_plugin::LabeledError> {
        // no need to 'match name {...}' because we only register mers-nu and nothing else.
        let source: String = call.req(0)?;
        let source_span = Span::unknown(); // source.span;
                                           // let source = source.item;
        let mut file = if call.has_flag("execute") {
            parsing::file::File::new(source, PathBuf::new())
        } else {
            parsing::file::File::new(
                match fs::read_to_string(&source) {
                    Ok(v) => v,
                    Err(e) => {
                        return Ok(Value::Error {
                            error: Box::new(ShellError::FileNotFound(source_span)),
                        })
                    }
                },
                source.into(),
            )
        };
        Ok(match parsing::parse::parse(&mut file) {
            Ok(code) => {
                let args = match call.opt(1)? {
                    Some(v) => v,
                    _ => vec![],
                };
                fn to_nu_val(val: VData, info: &GlobalScriptInfo) -> Value {
                    let span = Span::unknown();
                    match val.data {
                        VDataEnum::Bool(val) => Value::Bool { val, span },
                        VDataEnum::Int(val) => Value::Int {
                            val: val as _,
                            span,
                        },
                        VDataEnum::Float(val) => Value::Float { val, span },
                        VDataEnum::String(val) => Value::String { val, span },
                        VDataEnum::Tuple(vals) | VDataEnum::List(_, vals) => Value::List {
                            vals: vals.into_iter().map(|v| to_nu_val(v, info)).collect(),
                            span,
                        },
                        VDataEnum::Reference(r) => to_nu_val(
                            std::mem::replace(
                                &mut *r.lock().unwrap(),
                                VDataEnum::Tuple(vec![]).to(),
                            ),
                            info,
                        ),
                        VDataEnum::EnumVariant(variant, val) => {
                            let name = info
                                .enum_variants
                                .iter()
                                .find_map(|(name, id)| {
                                    if *id == variant {
                                        Some(name.to_owned())
                                    } else {
                                        None
                                    }
                                })
                                .unwrap();
                            Value::Record {
                                cols: vec![format!("Enum"), format!("Value")],
                                vals: vec![
                                    Value::String {
                                        val: name,
                                        span: span,
                                    },
                                    to_nu_val(*val, info),
                                ],
                                span,
                            }
                        }
                        VDataEnum::Function(func) => Value::Nothing { span },
                        VDataEnum::Thread(t, _) => to_nu_val(t.get(), info),
                    }
                }
                to_nu_val(code.run(args), code.info().as_ref())
            }
            Err(e) => Value::Error {
                error: Box::new(ShellError::IncorrectValue {
                    msg: format!("Couldn't compile mers, error: {}", e.with_file(&file)),
                    span: source_span,
                }),
            },
        })
    }
}
