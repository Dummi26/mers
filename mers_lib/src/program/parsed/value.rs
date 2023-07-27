use crate::{data::Data, program};

use super::{CompInfo, MersStatement};

#[derive(Debug)]
pub struct Value(pub Data);

impl MersStatement for Value {
    fn has_scope(&self) -> bool {
        false
    }
    fn compile_custom(
        &self,
        info: &mut crate::info::Info<super::Local>,
        comp: CompInfo,
    ) -> Result<Box<dyn program::run::MersStatement>, String> {
        Ok(Box::new(program::run::value::Value {
            val: self.0.clone(),
        }))
    }
}
