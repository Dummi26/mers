use std::sync::Arc;

use super::MersStatement;

#[derive(Debug)]
pub struct Block {
    pub statements: Vec<Box<dyn MersStatement>>,
}
impl MersStatement for Block {
    fn run_custom(&self, info: &mut super::Info) -> crate::data::Data {
        self.statements
            .iter()
            .map(|s| s.run(info))
            .last()
            .unwrap_or_else(|| crate::data::Data::new(crate::data::tuple::Tuple(vec![])))
    }
    fn has_scope(&self) -> bool {
        true
    }
}
