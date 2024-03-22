use std::sync::{Arc, Mutex};

use crate::{
    data::{self, Data, MersType, Type},
    errors::{error_colors, CheckError, SourceRange},
};

use super::{Info, MersStatement};

#[derive(Debug)]
pub struct Try {
    pub pos_in_src: SourceRange,
    pub arg: Box<dyn MersStatement>,
    pub funcs: Vec<Box<dyn MersStatement>>,
    pub index_of_unused_try_statement: Mutex<Option<usize>>,
}

impl MersStatement for Try {
    fn check_custom(
        &self,
        info: &mut super::CheckInfo,
        init_to: Option<&Type>,
    ) -> Result<data::Type, CheckError> {
        if init_to.is_some() {
            return Err("can't init to statement type Try".to_string().into());
        }
        let mut t = Type::empty();
        let arg = self.arg.check(info, init_to)?;
        let funcs = self
            .funcs
            .iter()
            .map(|v| v.check(info, init_to))
            .collect::<Result<Vec<_>, CheckError>>()?;
        let mut index_lock = self.index_of_unused_try_statement.lock().unwrap();
        let mut unused_try_statements_lock = info.global.unused_try_statements.lock().unwrap();
        let used = if let Some(i) = *index_lock {
            &mut unused_try_statements_lock[i]
        } else {
            let my_index = unused_try_statements_lock.len();
            *index_lock = Some(my_index);
            unused_try_statements_lock.push((
                self.pos_in_src.clone(),
                self.funcs.iter().map(|v| Some(v.source_range())).collect(),
            ));
            &mut unused_try_statements_lock[my_index]
        };
        drop(index_lock);
        for arg in arg.subtypes_type().types.iter() {
            let mut found = false;
            let mut errs = vec![];
            for (i, func) in funcs.iter().enumerate() {
                let mut func_res = Type::empty();
                let mut func_err = None;
                for ft in func.types.iter() {
                    if let Some(ft) = ft.as_any().downcast_ref::<data::function::FunctionT>() {
                        match ft.o(&Type::newm(vec![Arc::clone(arg)])) {
                            Ok(res) => {
                                func_res.add_all(&res);
                            }
                            Err(e) => func_err = Some(e),
                        }
                    } else {
                        return Err(CheckError::new()
                            .msg(format!(
                                "try: #{} is not a function, type is {ft} within {func}.",
                                i + 1
                            ))
                            .src(vec![
                                (self.source_range(), None),
                                (
                                    self.funcs[i].source_range(),
                                    Some(error_colors::TryNotAFunction),
                                ),
                            ]));
                    }
                }
                if let Some(err) = func_err {
                    // can't use this function for this argument
                    errs.push(err);
                } else {
                    // found the function to use
                    used.1[i] = None;
                    found = true;
                    t.add_all(&func_res);
                    break;
                }
            }
            if !found {
                let mut err = CheckError::new()
                    .msg(format!(
                        "try: no function found for argument of type {arg}."
                    ))
                    .src(vec![(
                        self.pos_in_src.clone(),
                        Some(error_colors::TryNoFunctionFound),
                    )]);
                for (i, e) in errs.into_iter().enumerate() {
                    err = err.msg(format!("Error for function #{}:", i + 1)).err(e);
                }
                return Err(err);
            }
        }
        Ok(t)
    }
    fn run_custom(&self, info: &mut Info) -> Data {
        let arg = self.arg.run(info);
        let ar = arg.get();
        let a = ar.as_ref();
        let arg_type = a.as_type();
        let functions = self
            .funcs
            .iter()
            .map(|v| {
                v.run(info)
                    .get()
                    .as_any()
                    .downcast_ref::<data::function::Function>()
                    .unwrap()
                    .clone()
            })
            .collect::<Vec<_>>();
        let mut found = None;
        for (i, func) in functions.iter().enumerate() {
            match func.get_as_type().o(&arg_type) {
                Ok(_) => {
                    found = Some(i);
                    break;
                }
                Err(_) => (),
            }
        }
        drop(ar);
        functions[found.expect("try: no function found")].run(arg)
    }
    fn has_scope(&self) -> bool {
        true
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src.clone()
    }
    fn inner_statements(&self) -> Vec<&dyn MersStatement> {
        let mut o = vec![self.arg.as_ref()];
        o.extend(self.funcs.iter().map(|v| v.as_ref()));
        o
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
