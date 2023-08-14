use std::sync::Arc;

use crate::data::{self, Data, MersType, Type};

use super::{CheckError, MersStatement};

#[derive(Debug)]
pub struct If {
    pub condition: Box<dyn MersStatement>,
    pub on_true: Box<dyn MersStatement>,
    pub on_false: Option<Box<dyn MersStatement>>,
}

impl MersStatement for If {
    fn check_custom(
        &self,
        info: &mut super::CheckInfo,
        init_to: Option<&Type>,
    ) -> Result<data::Type, super::CheckError> {
        if init_to.is_some() {
            return Err(CheckError("can't init to statement type If".to_string()));
        }
        if !self
            .condition
            .check(info, None)?
            .is_included_in(&data::bool::BoolT)
        {
            return Err(CheckError(format!(
                "condition in an if-statement must return bool"
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
}
