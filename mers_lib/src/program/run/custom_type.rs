use std::{fmt::Debug, sync::Arc};

use colored::Colorize;

use crate::{
    data::{Data, Type},
    errors::{CheckError, SourceRange},
};

use super::{CheckInfo, Info, MersStatement};

pub struct CustomType {
    pub pos_in_src: SourceRange,
    pub name: String,
    pub source: Box<
        dyn Fn(
                &CheckInfo,
            ) -> Result<
                Result<
                    Arc<Type>,
                    Arc<dyn Fn(&str, &CheckInfo) -> Result<Arc<Type>, CheckError> + Send + Sync>,
                >,
                CheckError,
            > + Send
            + Sync,
    >,
}

impl MersStatement for CustomType {
    fn check_custom(
        &self,
        info: &mut CheckInfo,
        init_to: Option<&Type>,
    ) -> Result<Type, CheckError> {
        if init_to.is_some() {
            return Err("can't init to `type` statement".to_string().into());
        }
        let t = (self.source)(info);
        if self.name != "_" {
            info.scopes
                .last_mut()
                .unwrap()
                .types
                .insert(self.name.clone(), t?);
        } else {
            if let Err(e) = t {
                return Err(CheckError::new()
                    .msg(format!(
                        " {} {} {} (`[[_] := ...]` indicates that `...` must be type-correct)",
                        "<<".bright_red(),
                        "Custom type-test failed!".bright_red(),
                        ">>".bright_red(),
                    ))
                    .err(e));
            }
        }
        Ok(Type::empty_tuple())
    }
    fn run_custom(&self, _info: &mut Info) -> Result<Data, CheckError> {
        Ok(Data::empty_tuple())
    }
    fn has_scope(&self) -> bool {
        false
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

impl Debug for CustomType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<CustomType>")
    }
}
