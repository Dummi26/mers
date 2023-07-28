use std::{fs, path::PathBuf};

use nu_plugin::{serve_plugin, MsgPackSerializer, Plugin};
use nu_protocol::{PluginSignature, ShellError, Span, SyntaxShape, Value};

use crate::{
    lang::{
        fmtgs::FormatGs,
        global_info::GlobalScriptInfo,
        val_data::{VData, VDataEnum},
        val_type::VType,
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
                SyntaxShape::List(Box::new(SyntaxShape::OneOf(vec![
                    SyntaxShape::Boolean,
                    SyntaxShape::Int,
                    SyntaxShape::Decimal,
                    SyntaxShape::String,
                    SyntaxShape::List(Box::new(SyntaxShape::OneOf(vec![
                        SyntaxShape::Boolean,
                        SyntaxShape::Int,
                        SyntaxShape::Decimal,
                        SyntaxShape::String,
                        SyntaxShape::List(Box::new(SyntaxShape::OneOf(vec![
                            SyntaxShape::Boolean,
                            SyntaxShape::Int,
                            SyntaxShape::Decimal,
                            SyntaxShape::String,
                        ]))),
                    ]))),
                ]))),
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
        _name: &str,
        call: &nu_plugin::EvaluatedCall,
        _input: &nu_protocol::Value,
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
                    Err(_e) => {
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
                    Some(v) => {
                        fn to_mers_val(v: Vec<Value>, info: &GlobalScriptInfo) -> Vec<VData> {
                            v.into_iter()
                                .map(|v| {
                                    match v {
                                        Value::Bool { val, .. } => VDataEnum::Bool(val),
                                        Value::Int { val, .. } => VDataEnum::Int(val as _),
                                        Value::Float { val, .. } => VDataEnum::Float(val),
                                        Value::String { val, .. } => VDataEnum::String(val),
                                        Value::List { vals, .. } => {
                                            let mut t = VType::empty();
                                            let mut vs = Vec::with_capacity(vals.len());
                                            for v in to_mers_val(vals, info) {
                                                t.add_types(v.out(), info);
                                                vs.push(v);
                                            }
                                            VDataEnum::List(t, vs)
                                        }
                                        _ => unreachable!("invalid arg type"),
                                    }
                                    .to()
                                })
                                .collect()
                        }
                        if let Value::List { vals, .. } = v {
                            to_mers_val(vals, &code.info)
                        } else {
                            unreachable!("args not a list")
                        }
                    }
                    _ => vec![],
                };
                fn to_nu_val(val: &VData, info: &GlobalScriptInfo) -> Value {
                    let span = Span::unknown();
                    val.operate_on_data_immut(|val| match val {
                        VDataEnum::Bool(val) => Value::Bool { val: *val, span },
                        VDataEnum::Int(val) => Value::Int {
                            val: *val as _,
                            span,
                        },
                        VDataEnum::Float(val) => Value::Float { val: *val, span },
                        VDataEnum::String(val) => Value::String {
                            val: val.to_owned(),
                            span,
                        },
                        VDataEnum::Tuple(vals) | VDataEnum::List(_, vals) => Value::List {
                            vals: vals.iter().map(|v| to_nu_val(v, info)).collect(),
                            span,
                        },
                        VDataEnum::Reference(r) => to_nu_val(r, info),
                        VDataEnum::EnumVariant(variant, val) => {
                            let name = info
                                .enum_variants
                                .iter()
                                .find_map(|(name, id)| {
                                    if *id == *variant {
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
                                    to_nu_val(val, info),
                                ],
                                span,
                            }
                        }
                        VDataEnum::Function(_func) => Value::Nothing { span },
                        VDataEnum::Thread(t, _) => to_nu_val(&t.get(), info),
                    })
                }
                to_nu_val(&code.run(args), &code.info)
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
