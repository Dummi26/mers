use std::sync::Arc;

use crate::{
    data::{
        self,
        object::ObjectT,
        tuple::{Tuple, TupleT},
        Data, MersDataWInfo, MersTypeWInfo, Type,
    },
    errors::{CheckError, EColor, SourceRange},
};

use super::MersStatement;

#[derive(Debug)]
pub struct FieldChain {
    pub pos_in_src: SourceRange,
    pub object: Box<dyn MersStatement>,
    pub args: Option<(Vec<Box<dyn MersStatement>>, SourceRange)>,
    pub field_str: String,
    pub field_pos: SourceRange,
    pub field: usize,
}
impl MersStatement for FieldChain {
    fn check_custom(
        &self,
        info: &mut super::CheckInfo,
        init_to: Option<&Type>,
    ) -> Result<data::Type, super::CheckError> {
        if init_to.is_some() {
            return Err("can't init to statement type Field".to_string().into());
        }
        let object = self.object.check(info, init_to)?;
        let mut o = Type::empty();
        for arg in object.types.iter() {
            let iter;
            let iter = if let Some(t) = arg.is_reference_to() {
                t.types.iter()
            } else {
                iter = vec![Arc::clone(arg)];
                iter.iter()
            };
            let arg = Type::newm(vec![Arc::clone(arg)]);
            let arg = if let Some((more_args, _)) = &self.args {
                let mut args = vec![arg];
                for res in more_args.iter().map(|arg| arg.check(info, None)) {
                    args.push(res?);
                }
                Type::new(TupleT(args))
            } else {
                arg
            };
            for t in iter {
                if let Some(t) = t.as_any().downcast_ref::<ObjectT>() {
                    if let Some(func) = t.get(self.field) {
                        o.add_all(&super::chain::check(
                            &arg,
                            func,
                            info,
                            self.pos_in_src.clone(),
                            self.object.source_range(),
                            self.field_pos.clone(),
                            None,
                        )?);
                    } else {
                        return Err(CheckError::new().msg(vec![
                            ("can't get field ".to_owned(), None),
                            (self.field_str.clone(), Some(EColor::ObjectField)),
                            (" of object ".to_owned(), None),
                            (t.with_info(info).to_string(), Some(EColor::InitFrom)),
                        ]));
                    }
                } else {
                    return Err(CheckError::new().msg(vec![
                        ("can't get field ".to_owned(), None),
                        (self.field_str.clone(), Some(EColor::ObjectField)),
                        (" of non-object type ".to_owned(), None),
                        (arg.with_info(info).to_string(), Some(EColor::InitFrom)),
                    ]));
                }
            }
        }
        Ok(o)
    }
    fn run_custom(&self, info: &mut super::Info) -> Result<Data, CheckError> {
        let object = self.object.run(info)?;
        let func = {
            let object_lock = object.get();
            let obj_ref;
            let obj_in_ref;
            let object = if let Some(o) =
                if let Some(o) = object_lock.as_any().downcast_ref::<data::object::Object>() {
                    Some(o)
                } else if let Some(r) = object_lock
                    .as_any()
                    .downcast_ref::<data::reference::Reference>()
                {
                    obj_ref = r.read();
                    obj_in_ref = obj_ref.get();
                    obj_in_ref.as_any().downcast_ref::<data::object::Object>()
                } else {
                    None
                } {
                o
            } else {
                Err(format!(
                    "couldn't extract field {} from non-object and non-&object value {}",
                    self.field_str,
                    object_lock.with_info(info)
                ))?
            };
            let func = object
                .get(self.field)
                .ok_or_else(|| {
                    format!(
                        "couldn't extract field {} from object {}",
                        self.field_str,
                        object.with_info(info)
                    )
                })?
                .clone();
            func
        };
        let arg = if let Some((more_args, _)) = &self.args {
            let mut args = vec![object];
            for res in more_args.iter().map(|arg| arg.run(info)) {
                args.push(res?);
            }
            Data::new(Tuple::from(args))
        } else {
            object
        };
        super::chain::run(
            arg,
            func,
            info,
            self.pos_in_src.clone(),
            self.object.source_range(),
            self.field_pos.clone(),
            None,
        )
    }
    fn has_scope(&self) -> bool {
        false
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src.clone()
    }
    fn inner_statements(&self) -> Vec<&dyn MersStatement> {
        let mut o = vec![&*self.object];
        if let Some((args, _)) = &self.args {
            o.extend(args.iter().map(|v| &**v));
        }
        o
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
