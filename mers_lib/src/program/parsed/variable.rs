use crate::{
    errors::{error_colors, CheckError, SourceRange},
    info::Local,
    program::{self},
};

use super::{CompInfo, MersStatement};

#[derive(Debug)]
pub struct Variable {
    pub pos_in_src: SourceRange,
    pub is_ref: bool,
    pub var: String,
}

impl MersStatement for Variable {
    fn has_scope(&self) -> bool {
        false
    }
    fn compile_custom(
        &self,
        info: &mut crate::info::Info<super::Local>,
        comp: CompInfo,
    ) -> Result<Box<dyn program::run::MersStatement>, CheckError> {
        if comp.is_init {
            info.init_var(
                self.var.clone(),
                (
                    info.scopes.len() - 1,
                    info.scopes.last().unwrap().vars_count,
                ),
            )
        }
        Ok(Box::new(program::run::variable::Variable {
            pos_in_src: self.pos_in_src,
            is_init: comp.is_init,
            is_ref: comp.is_init || self.is_ref,
            var: if let Some(v) = info.get_var(&self.var) {
                *v
            } else {
                return Err(CheckError::new()
                    .src(vec![(self.pos_in_src, Some(error_colors::UnknownVariable))])
                    .msg(format!("No variable named '{}' found!", self.var)));
            },
        }))
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src
    }
}
