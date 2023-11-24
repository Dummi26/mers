use std::sync::{Arc, Mutex};

use colored::Colorize;

use crate::{
    data::{self, Data},
    errors::{error_colors, CheckError, SourceRange},
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
        let compiled: Arc<Box<dyn crate::program::run::MersStatement>> =
            match self.include.compile(&mut info.duplicate(), comp) {
                Ok(v) => Arc::new(v),
                Err(e) => {
                    return Err(CheckError::new()
                        .src(vec![(
                            self.pos_in_src.clone(),
                            Some(error_colors::HashIncludeErrorInIncludedFile),
                        )])
                        .msg(
                            "Error in #include! (note: inner errors may refer to a different file)"
                                .color(error_colors::HashIncludeErrorInIncludedFile)
                                .to_string(),
                        )
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
                    info: Arc::new(info::Info::neverused()),
                    info_check: Arc::new(Mutex::new(info::Info::neverused())),
                    out: Arc::new(move |_, i| compiled.check(&mut i.duplicate(), None)),
                    run: Arc::new(move |_, i| compiled2.run(&mut i.duplicate())),
                },
            }),
            as_part_of_include: Some(self.inner_src.clone()),
        }))
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src.clone()
    }
}
