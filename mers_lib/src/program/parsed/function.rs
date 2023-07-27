use std::sync::Arc;

use crate::{
    data::{self, Data},
    info::Local,
    program,
};

use super::{CompInfo, MersStatement};

#[derive(Debug)]
pub struct Function {
    pub arg: Box<dyn MersStatement>,
    pub run: Box<dyn MersStatement>,
}

impl MersStatement for Function {
    fn has_scope(&self) -> bool {
        // TODO: what???
        false
    }
    fn compile_custom(
        &self,
        info: &mut crate::info::Info<super::Local>,
        mut comp: CompInfo,
    ) -> Result<Box<dyn program::run::MersStatement>, String> {
        comp.is_init = true;
        let arg = self.arg.compile(info, comp)?;
        comp.is_init = false;
        let run = self.run.compile(info, comp)?;
        Ok(Box::new(program::run::function::Function {
            func_no_info: data::function::Function {
                info: program::run::Info::neverused(),
                out: Arc::new(|_i| todo!()),
                run: Arc::new(move |i, info| {
                    data::defs::assign(i, &arg.run(info));
                    run.run(info)
                }),
            },
        }))
    }
}
