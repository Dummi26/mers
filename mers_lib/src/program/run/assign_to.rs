use crate::data;

use super::MersStatement;

#[derive(Debug)]
pub struct AssignTo {
    pub target: Box<dyn MersStatement>,
    pub source: Box<dyn MersStatement>,
}

impl MersStatement for AssignTo {
    fn run_custom(&self, info: &mut super::Info) -> crate::data::Data {
        let source = self.source.run(info);
        let target = self.target.run(info);
        data::defs::assign(source, &target);
        target
    }
    fn has_scope(&self) -> bool {
        false
    }
}
