use crate::parsing::SourcePos;
use crate::program;

use super::{CompInfo, MersStatement};

#[derive(Debug)]
pub struct InitTo {
    pub pos_in_src: SourcePos,
    pub target: Box<dyn MersStatement>,
    pub source: Box<dyn MersStatement>,
}

impl MersStatement for InitTo {
    fn has_scope(&self) -> bool {
        false
    }
    fn compile_custom(
        &self,
        info: &mut crate::info::Info<super::Local>,
        mut comp: CompInfo,
    ) -> Result<Box<dyn crate::program::run::MersStatement>, String> {
        comp.is_init = true;
        let target = self.target.compile(info, comp)?;
        comp.is_init = false;
        let source = self.source.compile(info, comp)?;
        Ok(Box::new(program::run::assign_to::AssignTo {
            pos_in_src: self.pos_in_src,
            is_init: true,
            target,
            source,
        }))
    }
}
