use crate::program;

use super::{CompInfo, MersStatement};

#[derive(Debug)]
pub struct Loop {
    pub inner: Box<dyn MersStatement>,
}

impl MersStatement for Loop {
    fn has_scope(&self) -> bool {
        true
    }
    fn compile_custom(
        &self,
        info: &mut crate::info::Info<super::Local>,
        comp: CompInfo,
    ) -> Result<Box<dyn program::run::MersStatement>, String> {
        Ok(Box::new(program::run::r#loop::Loop {
            inner: self.inner.compile(info, comp)?,
        }))
    }
}
