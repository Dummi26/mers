use std::sync::{Arc, Mutex};

use crate::{
    data,
    errors::{CheckError, SourceRange},
    program::{self, run::CheckInfo},
};

use super::{CompInfo, MersStatement};

#[derive(Debug)]
pub struct Function {
    pub pos_in_src: SourceRange,
    pub arg: Box<dyn MersStatement>,
    pub run: Box<dyn MersStatement>,
}

impl MersStatement for Function {
    fn has_scope(&self) -> bool {
        // TODO: what???
        true
    }
    fn compile_custom(
        &self,
        info: &mut crate::info::Info<super::Local>,
        mut comp: CompInfo,
    ) -> Result<Box<dyn program::run::MersStatement>, CheckError> {
        comp.is_init = true;
        let arg_target = Arc::new(self.arg.compile(info, comp)?);
        comp.is_init = false;
        let run = Arc::new(self.run.compile(info, comp)?);
        let arg2 = Arc::clone(&arg_target);
        let run2 = Arc::clone(&run);
        let arg3 = Arc::clone(&arg_target);
        let run3 = Arc::clone(&run);
        Ok(Box::new(program::run::function::Function {
            pos_in_src: self.pos_in_src.clone(),
            func_no_info: data::function::Function {
                info: Arc::new(program::run::Info::neverused()),
                info_check: Arc::new(Mutex::new(CheckInfo::neverused())),
                out: Arc::new(move |a, i| {
                    arg2.check(i, Some(a))?;
                    Ok(run2.check(i, None)?)
                }),
                run: Arc::new(move |arg, info| {
                    data::defs::assign(&arg, &arg_target.run(info)?);
                    run.run(info)
                }),
                inner_statements: Some((arg3, run3)),
            },
        }))
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src.clone()
    }
    fn inner_statements(&self) -> Vec<&dyn MersStatement> {
        vec![self.arg.as_ref(), self.run.as_ref()]
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
