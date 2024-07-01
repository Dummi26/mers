use std::sync::{Arc, Mutex};

use crate::{
    data::{self, Data, Type},
    errors::{CheckError, EColor, SourceRange},
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
        let my_index = if let Some(i) = *index_lock {
            i
        } else {
            let my_index = unused_try_statements_lock.len();
            *index_lock = Some(my_index);
            unused_try_statements_lock.push((
                self.pos_in_src.clone(),
                self.funcs.iter().map(|v| Some(v.source_range())).collect(),
            ));
            my_index
        };
        drop(unused_try_statements_lock);
        drop(index_lock);
        for arg in arg.subtypes_type().types.iter() {
            let mut found = false;
            let mut errs = vec![];
            for (i, func) in funcs.iter().enumerate() {
                let mut func_res = Type::empty();
                let mut func_err = None;
                for ft in func.types.iter() {
                    if let Some(ft) = ft.executable() {
                        match ft.o(&Type::newm(vec![Arc::clone(arg)])) {
                            Ok(res) => {
                                func_res.add_all(&res);
                            }
                            Err(e) => func_err = Some(e),
                        }
                    } else {
                        return Err(CheckError::new()
                            .msg_str(format!(
                                "try: #{} is not a function, type is {ft} within {func}.",
                                i + 1
                            ))
                            .src(vec![
                                (self.source_range(), None),
                                (self.funcs[i].source_range(), Some(EColor::TryNotAFunction)),
                            ]));
                    }
                }
                if let Some(err) = func_err {
                    // can't use this function for this argument
                    errs.push(err);
                } else {
                    // found the function to use
                    info.global.unused_try_statements.lock().unwrap()[my_index].1[i] = None;
                    found = true;
                    t.add_all(&func_res);
                    break;
                }
            }
            if !found {
                let mut err = CheckError::new()
                    .msg_str(format!(
                        "try: no function found for argument of type {arg}."
                    ))
                    .src(vec![(
                        self.pos_in_src.clone(),
                        Some(EColor::TryNoFunctionFound),
                    )]);
                for (i, e) in errs.into_iter().enumerate() {
                    err = err
                        .msg_str(format!("Error for function #{}:", i + 1))
                        .err(e);
                }
                return Err(err);
            }
        }
        Ok(t)
    }
    fn run_custom(&self, info: &mut Info) -> Result<Data, CheckError> {
        let arg = self.arg.run(info)?;
        let ar = arg.get();
        let a = ar.as_ref();
        let arg_type = a.as_type();
        for func in self.funcs.iter() {
            let func = func.run(info)?;
            let func = func.get();
            match func.executable().map(|func| func.o(&arg_type)) {
                Some(Ok(_)) => {
                    drop(ar);
                    return func.execute(arg).unwrap();
                }
                None | Some(Err(_)) => (),
            }
        }
        panic!("try: no function found")
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
