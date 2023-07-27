use crate::data::{self, Data, MersData};

use super::MersStatement;

#[derive(Debug)]
pub struct Function {
    pub func_no_info: data::function::Function,
}

impl MersStatement for Function {
    fn has_scope(&self) -> bool {
        false
    }
    fn run_custom(&self, info: &mut super::Info) -> Data {
        Data::new(self.func_no_info.with_info(info.clone()))
    }
}
