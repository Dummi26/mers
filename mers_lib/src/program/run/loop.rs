use crate::data::MersData;

use super::MersStatement;

#[derive(Debug)]
pub struct Loop {
    pub inner: Box<dyn MersStatement>,
}

impl MersStatement for Loop {
    fn run_custom(&self, info: &mut super::Info) -> crate::data::Data {
        loop {
            if let Some(break_val) = self.inner.run(info).get().matches() {
                break break_val;
            }
        }
    }
    fn has_scope(&self) -> bool {
        true
    }
}
