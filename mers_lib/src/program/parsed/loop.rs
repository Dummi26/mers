use crate::{
    errors::{CheckError, SourceRange},
    program::{self},
};

use super::{CompInfo, MersStatement};

#[derive(Debug)]
pub struct Loop {
    pub pos_in_src: SourceRange,
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
    ) -> Result<Box<dyn program::run::MersStatement>, CheckError> {
        Ok(Box::new(program::run::r#loop::Loop {
            pos_in_src: self.pos_in_src.clone(),
            inner: self.inner.compile(info, comp)?,
        }))
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src.clone()
    }
    fn inner_statements(&self) -> Vec<&dyn MersStatement> {
        vec![self.inner.as_ref()]
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
