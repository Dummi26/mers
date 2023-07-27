use crate::data::Type;

use super::MersStatement;

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
    fn run_custom(&self, info: &mut super::Info) -> crate::data::Data {
        todo!("switch")
    }
}
