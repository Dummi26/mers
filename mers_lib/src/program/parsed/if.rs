use crate::program::{self, run::SourceRange};

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
    ) -> Result<Box<dyn program::run::MersStatement>, String> {
        Ok(Box::new(program::run::r#if::If {
            pos_in_src: self.pos_in_src,
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
        self.pos_in_src
    }
}
