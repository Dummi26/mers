use crate::{
    errors::{CheckError, SourceRange},
    program::{self},
};

use super::{CompInfo, MersStatement};

#[derive(Debug)]
pub struct AssignTo {
    pub pos_in_src: SourceRange,
    pub source: Box<dyn MersStatement>,
    pub target: Box<dyn MersStatement>,
}

impl MersStatement for AssignTo {
    fn has_scope(&self) -> bool {
        false
    }
    fn compile_custom(
        &self,
        info: &mut crate::info::Info<super::Local>,
        comp: CompInfo,
    ) -> Result<Box<dyn program::run::MersStatement>, CheckError> {
        Ok(Box::new(program::run::assign_to::AssignTo {
            pos_in_src: self.pos_in_src.clone(),
            is_init: false,
            source: self.source.compile(info, comp)?,
            target: self.target.compile(info, comp)?,
        }))
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src.clone()
    }
}
