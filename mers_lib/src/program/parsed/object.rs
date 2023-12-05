use crate::{
    errors::{CheckError, SourceRange},
    info,
    program::{self},
};

use super::{CompInfo, MersStatement};

#[derive(Debug)]
pub struct Object {
    pub pos_in_src: SourceRange,
    pub elems: Vec<(String, Box<dyn MersStatement>)>,
}
impl MersStatement for Object {
    fn has_scope(&self) -> bool {
        false
    }
    fn compile_custom(
        &self,
        info: &mut info::Info<super::Local>,
        comp: CompInfo,
    ) -> Result<Box<dyn program::run::MersStatement>, CheckError> {
        Ok(Box::new(program::run::object::Object {
            pos_in_src: self.pos_in_src.clone(),
            elems: self
                .elems
                .iter()
                .map(|(n, v)| -> Result<_, CheckError> { Ok((n.clone(), v.compile(info, comp)?)) })
                .collect::<Result<Vec<_>, _>>()?,
        }))
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src.clone()
    }
    fn inner_statements(&self) -> Vec<&dyn MersStatement> {
        self.elems.iter().map(|(_, s)| s.as_ref()).collect()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
