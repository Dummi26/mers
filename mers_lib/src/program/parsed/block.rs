use crate::{info, parsing::SourcePos, program};

use super::{CompInfo, MersStatement};

#[derive(Debug)]
pub struct Block {
    pub pos_in_src: SourcePos,
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
    ) -> Result<Box<dyn program::run::MersStatement>, String> {
        Ok(Box::new(program::run::block::Block {
            pos_in_src: self.pos_in_src,
            statements: self
                .statements
                .iter()
                .map(|v| v.compile(info, comp))
                .collect::<Result<Vec<_>, _>>()?,
        }))
    }
}