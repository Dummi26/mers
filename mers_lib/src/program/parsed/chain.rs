use crate::{
    errors::{CheckError, SourceRange},
    info, program,
};

use super::{CompInfo, MersStatement};

#[derive(Debug)]
pub struct Chain {
    pub pos_in_src: SourceRange,
    pub first: Box<dyn MersStatement>,
    pub chained: Box<dyn MersStatement>,
}
impl MersStatement for Chain {
    fn has_scope(&self) -> bool {
        false
    }
    fn compile_custom(
        &self,
        info: &mut info::Info<super::Local>,
        comp: CompInfo,
    ) -> Result<Box<dyn program::run::MersStatement>, CheckError> {
        Ok(Box::new(program::run::chain::Chain {
            pos_in_src: self.pos_in_src.clone(),
            first: self.first.compile(info, comp)?,
            chained: self.chained.compile(info, comp)?,
            as_part_of_include: None,
        }))
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src.clone()
    }
    fn inner_statements(&self) -> Vec<&dyn MersStatement> {
        vec![self.first.as_ref(), self.chained.as_ref()]
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
