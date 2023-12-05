use crate::{
    errors::{CheckError, SourceRange},
    program::{self},
};

use super::{CompInfo, MersStatement};

#[derive(Debug)]
pub struct If {
    pub pos_in_src: SourceRange,
    pub condition: Box<dyn MersStatement>,
    pub on_true: Box<dyn MersStatement>,
    pub on_false: Option<Box<dyn MersStatement>>,
}

impl MersStatement for If {
    fn has_scope(&self) -> bool {
        true
    }
    fn compile_custom(
        &self,
        info: &mut crate::info::Info<super::Local>,
        comp: CompInfo,
    ) -> Result<Box<dyn program::run::MersStatement>, CheckError> {
        Ok(Box::new(program::run::r#if::If {
            pos_in_src: self.pos_in_src.clone(),
            condition: self.condition.compile(info, comp)?,
            on_true: self.on_true.compile(info, comp)?,
            on_false: if let Some(v) = &self.on_false {
                Some(v.compile(info, comp)?)
            } else {
                None
            },
        }))
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src.clone()
    }
    fn inner_statements(&self) -> Vec<&dyn MersStatement> {
        if let Some(on_false) = &self.on_false {
            vec![
                self.condition.as_ref(),
                self.on_true.as_ref(),
                on_false.as_ref(),
            ]
        } else {
            vec![self.condition.as_ref(), self.on_true.as_ref()]
        }
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
