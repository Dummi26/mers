use crate::data::{self, Data};

use super::MersStatement;

#[derive(Debug)]
pub struct Tuple {
    pub elems: Vec<Box<dyn MersStatement>>,
}
impl MersStatement for Tuple {
    fn run_custom(&self, info: &mut super::Info) -> crate::data::Data {
        Data::new(data::tuple::Tuple(
            self.elems.iter().map(|s| s.run(info)).collect(),
        ))
    }
    fn has_scope(&self) -> bool {
        false
    }
}
