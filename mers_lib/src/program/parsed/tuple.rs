use crate::{info, program};

use super::{CompInfo, MersStatement};

#[derive(Debug)]
pub struct Tuple {
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
            elems: self
                .elems
                .iter()
                .map(|v| v.compile(info, comp))
                .collect::<Result<Vec<_>, _>>()?,
        }))
    }
}
