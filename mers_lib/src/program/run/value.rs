use crate::{
    data::{Data, Type},
    errors::SourceRange,
};

use super::{CheckInfo, MersStatement};

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
        _info: &mut CheckInfo,
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
        self.pos_in_src.clone()
    }
    fn inner_statements(&self) -> Vec<&dyn MersStatement> {
        vec![]
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
