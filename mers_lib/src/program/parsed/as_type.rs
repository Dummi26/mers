use crate::{
    errors::{CheckError, SourceRange},
    parsing::types::ParsedType,
    program::{self},
};

use super::{CompInfo, MersStatement};

#[derive(Debug)]
pub struct AsType {
    pub pos_in_src: SourceRange,
    pub statement: Box<dyn MersStatement>,
    pub as_type: Vec<ParsedType>,
    pub type_pos_in_src: SourceRange,
    pub expand_type: bool,
}

impl MersStatement for AsType {
    fn has_scope(&self) -> bool {
        false
    }
    fn compile_custom(
        &self,
        info: &mut crate::info::Info<super::Local>,
        comp: CompInfo,
    ) -> Result<Box<dyn program::run::MersStatement>, CheckError> {
        Ok(Box::new(program::run::as_type::AsType {
            pos_in_src: self.pos_in_src,
            statement: self.statement.compile(info, comp)?,
            as_type: self.as_type.clone(),
            type_pos_in_src: self.type_pos_in_src,
            expand_type: self.expand_type,
        }))
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src
    }
}
