use crate::{
    data::{Data, Type},
    parsing::SourcePos,
};

use super::{CheckError, MersStatement};

#[derive(Debug)]
pub struct Value {
    pub pos_in_src: SourcePos,
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
            return Err(CheckError("can't init to statement type Value".to_string()));
        }
        Ok(self.val.get().as_type())
    }
    fn run_custom(&self, _info: &mut super::Info) -> Data {
        self.val.clone()
    }
    fn pos_in_src(&self) -> &SourcePos {
        &self.pos_in_src
    }
}