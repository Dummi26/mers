use std::sync::Arc;

use crate::{
    data::{self, Data, MersData, Type},
    parsing::SourcePos,
};

use super::{CheckError, MersStatement};

#[derive(Debug)]
pub struct Function {
    pub pos_in_src: SourcePos,
    pub func_no_info: data::function::Function,
}

impl MersStatement for Function {
    fn check_custom(
        &self,
        info: &mut super::CheckInfo,
        init_to: Option<&Type>,
    ) -> Result<data::Type, super::CheckError> {
        if init_to.is_some() {
            return Err(CheckError(
                "can't init to statement type Function".to_string(),
            ));
        }
        self.func_no_info.with_info_check(info.clone());
        Ok(self.func_no_info.as_type())
    }
    fn run_custom(&self, info: &mut super::Info) -> Data {
        Data::new(self.func_no_info.with_info_run(Arc::new(info.clone())))
    }
    fn has_scope(&self) -> bool {
        true
    }
    fn pos_in_src(&self) -> &SourcePos {
        &self.pos_in_src
    }
}
