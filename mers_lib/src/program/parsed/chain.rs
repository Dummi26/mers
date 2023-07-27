use crate::{info, program};

use super::{CompInfo, MersStatement};

#[derive(Debug)]
pub struct Chain {
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
    ) -> Result<Box<dyn program::run::MersStatement>, String> {
        Ok(Box::new(program::run::chain::Chain {
            first: self.first.compile(info, comp)?,
            chained: self.chained.compile(info, comp)?,
        }))
    }
}
