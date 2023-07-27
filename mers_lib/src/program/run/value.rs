use crate::data::{Data, MersData};

use super::MersStatement;

#[derive(Debug)]
pub struct Value {
    pub val: Data,
}

impl MersStatement for Value {
    fn has_scope(&self) -> bool {
        false
    }
    fn run_custom(&self, info: &mut super::Info) -> Data {
        self.val.clone()
    }
}
