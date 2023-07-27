use crate::data::Type;

use super::{CompInfo, MersStatement};

#[derive(Debug)]
pub struct Switch {
    source: Box<dyn MersStatement>,
    arms: Vec<SwitchArm>,
}

#[derive(Debug)]
pub struct SwitchArm {
    requires_type: Type,
}

impl MersStatement for Switch {
    fn has_scope(&self) -> bool {
        true
    }
    fn compile_custom(
        &self,
        info: &mut crate::info::Info<super::Local>,
        comp: CompInfo,
    ) -> Result<Box<dyn crate::program::run::MersStatement>, String> {
        todo!()
    }
}
