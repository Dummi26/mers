use std::sync::{Arc, Mutex};

use crate::{
    data::{self, Data},
    errors::{CheckError, EColor, SourceRange},
    info::{self, Local},
    parsing::Source,
    program::{self},
};

use super::{CompInfo, MersStatement};

#[derive(Debug)]
pub struct IncludeMers {
    pub pos_in_src: SourceRange,
    pub include: Box<dyn MersStatement>,
    pub inner_src: Source,
}
impl MersStatement for IncludeMers {
    fn has_scope(&self) -> bool {
        true
    }
    fn compile_custom(
        &self,
        info: &mut info::Info<super::Local>,
        comp: CompInfo,
    ) -> Result<Box<dyn program::run::MersStatement>, CheckError> {
        let mut inc_info = info.duplicate();
        inc_info.global.enable_hooks = false;
        let compiled: Arc<Box<dyn crate::program::run::MersStatement>> =
            match self.include.compile(&mut inc_info, comp) {
                Ok(v) => Arc::new(v),
                Err(e) => {
                    return Err(CheckError::new()
                        .src(vec![(
                            self.pos_in_src.clone(),
                            Some(EColor::HashIncludeErrorInIncludedFile),
                        )])
                        .msg(vec![(
                            "Error in #include! (note: inner errors may refer to a different file)"
                                .to_owned(),
                            Some(EColor::HashIncludeErrorInIncludedFile),
                        )])
                        .err_with_diff_src(e))
                }
            };
        let compiled2 = Arc::clone(&compiled);
        Ok(Box::new(program::run::chain::Chain {
            pos_in_src: self.pos_in_src.clone(),
            first: Box::new(program::run::value::Value {
                pos_in_src: self.pos_in_src.clone(),
                val: Data::empty_tuple(),
            }),
            chained: Box::new(program::run::function::Function {
                pos_in_src: self.pos_in_src.clone(),
                func_no_info: data::function::Function {
                    info: info::Info::neverused(),
                    info_check: Arc::new(Mutex::new(info::Info::neverused())),
                    out: Arc::new(move |_, i| compiled.check(&mut i.duplicate(), None)),
                    run: Arc::new(move |_, i| compiled2.run(&mut i.duplicate())),
                    inner_statements: None,
                },
            }),
            as_part_of_include: Some(self.inner_src.clone()),
        }))
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
