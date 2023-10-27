use crate::program::{self, run::SourceRange};

use super::{CompInfo, MersStatement};

#[derive(Debug)]
pub struct AssignTo {
    pub pos_in_src: SourceRange,
    pub target: Box<dyn MersStatement>,
    pub source: Box<dyn MersStatement>,
}

impl MersStatement for AssignTo {
    fn has_scope(&self) -> bool {
        false
    }
    fn compile_custom(
        &self,
        info: &mut crate::info::Info<super::Local>,
        comp: CompInfo,
    ) -> Result<Box<dyn program::run::MersStatement>, String> {
        Ok(Box::new(program::run::assign_to::AssignTo {
            pos_in_src: self.pos_in_src,
            is_init: false,
            target: self.target.compile(info, comp)?,
            source: self.source.compile(info, comp)?,
        }))
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src
    }
}
