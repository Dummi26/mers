use crate::{
    errors::{CheckError, SourceRange},
    info,
    program::{self},
};

use super::{CompInfo, MersStatement};

#[derive(Debug)]
pub struct Block {
    pub pos_in_src: SourceRange,
    pub statements: Vec<Box<dyn MersStatement>>,
}
impl MersStatement for Block {
    fn has_scope(&self) -> bool {
        true
    }
    fn compile_custom(
        &self,
        info: &mut info::Info<super::Local>,
        comp: CompInfo,
    ) -> Result<Box<dyn program::run::MersStatement>, CheckError> {
        Ok(Box::new(program::run::block::Block {
            pos_in_src: self.pos_in_src.clone(),
            statements: self
                .statements
                .iter()
                .map(|v| v.compile(info, comp))
                .collect::<Result<Vec<_>, _>>()?,
        }))
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src.clone()
    }
}
