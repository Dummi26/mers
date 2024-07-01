use crate::{
    data::{self, Data, MersData, Type},
    errors::{CheckError, SourceRange},
    info::Local,
};

use super::MersStatement;

#[derive(Debug)]
pub struct Function {
    pub pos_in_src: SourceRange,
    pub func_no_info: data::function::Function,
}

impl MersStatement for Function {
    fn check_custom(
        &self,
        info: &mut super::CheckInfo,
        init_to: Option<&Type>,
    ) -> Result<data::Type, CheckError> {
        if init_to.is_some() {
            return Err("can't init to statement type Function".to_string().into());
        }
        self.func_no_info.with_info_check(info.clone());
        Ok(self.func_no_info.as_type())
    }
    fn run_custom(&self, info: &mut super::Info) -> Result<Data, CheckError> {
        Ok(Data::new(self.func_no_info.with_info_run(info.duplicate())))
    }
    fn has_scope(&self) -> bool {
        true
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src.clone()
    }
    fn inner_statements(&self) -> Vec<&dyn MersStatement> {
        if let Some((a, b)) = &self.func_no_info.inner_statements {
            vec![a.as_ref().as_ref(), b.as_ref().as_ref()]
        } else {
            vec![]
        }
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
