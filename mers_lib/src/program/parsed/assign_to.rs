use crate::{info::Local, program};

use super::{CompInfo, MersStatement};

#[derive(Debug)]
pub struct AssignTo {
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
        mut comp: CompInfo,
    ) -> Result<Box<dyn program::run::MersStatement>, String> {
        Ok(Box::new(program::run::assign_to::AssignTo {
            target: self.target.compile(info, comp)?,
            source: self.source.compile(info, comp)?,
        }))
    }
}
