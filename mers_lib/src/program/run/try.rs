use std::sync::Mutex;

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
        let mut arg = self.arg.check(info, init_to)?;
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
        'func_options: for (i, func) in funcs.iter().enumerate() {
            // TODO! handle the case where a function is a one-of-multiple type...
            if func.types.len() != 1 {
                return Err(format!(
                    "Try-statement requires clearly defined functions, but got type {}",
                    func.with_info(info)
                )
                .into());
            }
            for ft in func.types.iter() {
                if let Some(ft) = ft.executable() {
                    let using_func = |t: &mut Type, func_res: &Type| {
                        // found the function to use
                        info.global.unused_try_statements.lock().unwrap()[my_index].1[i] = None;
                        t.add_all(func_res);
                    };
                    match ft.o_try(&arg) {
                        Ok(res) => {
                            using_func(&mut t, &res);
                            arg = Type::empty();
                            break 'func_options;
                        }
                        Err((_, covered)) => {
                            for (t_in, t_out) in &covered {
                                arg.without_in_place_all(t_in);
                                using_func(&mut t, t_out);
                            }
                        }
                    }
                } else {
                    return Err(CheckError::new()
                        .msg_str(format!(
                            "try: #{} is not a function, type is {} within {}.",
                            i + 1,
                            ft.simplified_as_string(info),
                            func.simplify_for_display(info).with_info(info),
                        ))
                        .src(vec![
                            (self.source_range(), None),
                            (self.funcs[i].source_range(), Some(EColor::TryNotAFunction)),
                        ]));
                }
            }
        }
        if !arg.types.is_empty() {
            let mut err = CheckError::new()
                .msg_str(format!(
                    "try: uncovered argument type {}.",
                    arg.simplified_as_string(info)
                ))
                .src(vec![(
                    self.pos_in_src.clone(),
                    Some(EColor::TryNoFunctionFound),
                )]);
            for (i, func) in funcs.iter().enumerate() {
                err = err.msg_str(format!(
                    "Error for function #{} {}:",
                    i + 1,
                    func.with_info(info)
                ));
                for e in func
                    .types
                    .iter()
                    .filter_map(|t| t.executable().unwrap().o(&arg).err())
                {
                    err = err.err(e);
                }
            }
            return Err(err);
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
                    return func.execute(arg, &info.global).unwrap();
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
