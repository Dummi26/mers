use crate::{info, parsing::SourcePos, program};

use super::{CompInfo, MersStatement};

#[derive(Debug)]
pub struct Tuple {
    pub pos_in_src: SourcePos,
    pub elems: Vec<Box<dyn MersStatement>>,
}
impl MersStatement for Tuple {
    fn has_scope(&self) -> bool {
        false
    }
    fn compile_custom(
        &self,
        info: &mut info::Info<super::Local>,
        comp: CompInfo,
    ) -> Result<Box<dyn program::run::MersStatement>, String> {
        Ok(Box::new(program::run::tuple::Tuple {
            pos_in_src: self.pos_in_src,
            elems: self
                .elems
                .iter()
                .map(|v| v.compile(info, comp))
                .collect::<Result<Vec<_>, _>>()?,
        }))
    }
}