use crate::data::{Data, Type};

use super::{MersStatement, SourceRange};

#[derive(Debug)]
pub struct Value {
    pub pos_in_src: SourceRange,
    pub val: Data,
}

impl MersStatement for Value {
    fn has_scope(&self) -> bool {
        false
    }
    fn check_custom(
        &self,
        info: &mut super::CheckInfo,
        init_to: Option<&Type>,
    ) -> Result<crate::data::Type, super::CheckError> {
        if init_to.is_some() {
            return Err("can't init to statement type Value".to_string().into());
        }
        Ok(self.val.get().as_type())
    }
    fn run_custom(&self, _info: &mut super::Info) -> Data {
        self.val.clone()
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src
    }
}
