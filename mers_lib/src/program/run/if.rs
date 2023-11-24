use std::sync::Arc;

use colored::Colorize;

use crate::{
    data::{self, Data, MersType, Type},
    errors::{error_colors, CheckError, SourceRange},
};

use super::MersStatement;

#[derive(Debug)]
pub struct If {
    pub pos_in_src: SourceRange,
    pub condition: Box<dyn MersStatement>,
    pub on_true: Box<dyn MersStatement>,
    pub on_false: Option<Box<dyn MersStatement>>,
}

impl MersStatement for If {
    fn check_custom(
        &self,
        info: &mut super::CheckInfo,
        init_to: Option<&Type>,
    ) -> Result<data::Type, CheckError> {
        if init_to.is_some() {
            return Err("can't init to statement type If".to_string().into());
        }
        let cond_return_type = self.condition.check(info, None)?;
        if !cond_return_type.is_included_in(&data::bool::BoolT) {
            return Err(CheckError::new()
                .src(vec![
                    (self.pos_in_src.clone(), None),
                    (
                        self.condition.source_range(),
                        Some(error_colors::IfConditionNotBool),
                    ),
                ])
                .msg(format!(
                    "The {} in an if-statement must return bool, not {}",
                    "condition".color(error_colors::IfConditionNotBool),
                    cond_return_type
                        .to_string()
                        .color(error_colors::IfConditionNotBool),
                )));
        }
        let mut t = self.on_true.check(info, None)?;
        if let Some(f) = &self.on_false {
            t.add(Arc::new(f.check(info, None)?));
        } else {
            t.add(Arc::new(Type::empty_tuple()));
        }
        Ok(t)
    }
    fn run_custom(&self, info: &mut super::Info) -> crate::data::Data {
        if let Some(data::bool::Bool(true)) = self
            .condition
            .run(info)
            .get()
            .as_any()
            .downcast_ref::<data::bool::Bool>()
        {
            self.on_true.run(info)
        } else if let Some(on_false) = &self.on_false {
            on_false.run(info)
        } else {
            Data::empty_tuple()
        }
    }
    fn has_scope(&self) -> bool {
        true
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src.clone()
    }
}
