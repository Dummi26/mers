use crate::{info, program};

use super::{CompInfo, MersStatement};

#[derive(Debug)]
pub struct Block {
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
            statements: self
                .statements
                .iter()
                .map(|v| v.compile(info, comp))
                .collect::<Result<Vec<_>, _>>()?,
        }))
    }
}
