use std::sync::Mutex;

use crate::{
    errors::{CheckError, SourceRange},
    program::{self},
};

use super::{CompInfo, MersStatement};

#[derive(Debug)]
pub struct Try {
    pub pos_in_src: SourceRange,
    pub arg: Box<dyn MersStatement>,
    pub funcs: Vec<Box<dyn MersStatement>>,
}

impl MersStatement for Try {
    fn has_scope(&self) -> bool {
        true
    }
    fn compile_custom(
        &self,
        info: &mut crate::info::Info<super::Local>,
        comp: CompInfo,
    ) -> Result<Box<dyn program::run::MersStatement>, CheckError> {
        Ok(Box::new(program::run::r#try::Try {
            pos_in_src: self.pos_in_src.clone(),
            arg: self.arg.compile(info, comp)?,
            funcs: self
                .funcs
                .iter()
                .map(|v| v.compile(info, comp))
                .collect::<Result<_, _>>()?,
            index_of_unused_try_statement: Mutex::new(None),
        }))
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src.clone()
    }
    fn inner_statements(&self) -> Vec<&dyn MersStatement> {
        let mut o = vec![self.arg.as_ref()];
        o.extend(self.funcs.iter().map(|v| &**v));
        o
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
