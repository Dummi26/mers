use crate::{
    data::Data,
    errors::{CheckError, SourceRange},
    program,
};

use super::{CompInfo, MersStatement};

#[derive(Debug)]
pub struct Value {
    pub pos_in_src: SourceRange,
    pub data: Data,
}

impl MersStatement for Value {
    fn has_scope(&self) -> bool {
        false
    }
    fn compile_custom(
        &self,
        _info: &mut crate::info::Info<super::Local>,
        _comp: CompInfo,
    ) -> Result<Box<dyn program::run::MersStatement>, CheckError> {
        Ok(Box::new(program::run::value::Value {
            pos_in_src: self.pos_in_src.clone(),
            val: self.data.clone(),
        }))
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src.clone()
    }
}
