use crate::data::Data;

use super::MersStatement;

pub struct InitTo {
    pub target: Box<dyn MersStatement>,
    pub source: Box<dyn MersStatement>,
}

impl MersStatement for InitTo {
    fn has_scope(&self) -> bool {
        false
    }
    fn run(&self, info: &mut super::Info) -> Data {}
}
