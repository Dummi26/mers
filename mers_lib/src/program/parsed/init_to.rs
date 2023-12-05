use crate::{
    errors::{CheckError, SourceRange},
    program::{self},
};

use super::{CompInfo, MersStatement};

#[derive(Debug)]
pub struct InitTo {
    pub pos_in_src: SourceRange,
    pub source: Box<dyn MersStatement>,
    pub target: Box<dyn MersStatement>,
}

impl MersStatement for InitTo {
    fn has_scope(&self) -> bool {
        false
    }
    fn compile_custom(
        &self,
        info: &mut crate::info::Info<super::Local>,
        mut comp: CompInfo,
    ) -> Result<Box<dyn crate::program::run::MersStatement>, CheckError> {
        // source must be compiled BEFORE target!
        comp.is_init = false;
        let source = self.source.compile(info, comp)?;
        comp.is_init = true;
        let target = self.target.compile(info, comp)?;
        comp.is_init = false;
        Ok(Box::new(program::run::assign_to::AssignTo {
            pos_in_src: self.pos_in_src.clone(),
            is_init: true,
            source,
            target,
        }))
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src.clone()
    }
    fn inner_statements(&self) -> Vec<&dyn MersStatement> {
        vec![self.source.as_ref(), self.target.as_ref()]
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
