use std::sync::{Arc, Mutex};

use colored::Colorize;

use crate::{
    data::{self, Data},
    errors::{error_colors, CheckError, SourceRange},
    info::{self, Local},
    program::{self},
};

use super::{CompInfo, MersStatement};

#[derive(Debug)]
pub struct IncludeMers {
    pub pos_in_src: SourceRange,
    pub include: Box<dyn MersStatement>,
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
        let compiled: Arc<Box<dyn crate::program::run::MersStatement>> = match self.include.compile(info, comp) {
            Ok(v) => Arc::new(v),
            Err(e) => {
                return Err(CheckError::new()
                    .src(vec![(self.pos_in_src, Some(error_colors::HashIncludeErrorInIncludedFile))])
                    .msg("Error in inner mers statement! (note: inner errors may refer to a different file)".color(error_colors::HashIncludeErrorInIncludedFile).to_string())
                .err(e))
            }
        };
        let compiled2 = Arc::clone(&compiled);
        Ok(Box::new(program::run::chain::Chain {
            pos_in_src: self.pos_in_src,
            first: Box::new(program::run::value::Value {
                pos_in_src: self.pos_in_src,
                val: Data::empty_tuple(),
            }),
            chained: Box::new(program::run::function::Function {
                pos_in_src: self.pos_in_src,
                func_no_info: data::function::Function {
                    info: Arc::new(info::Info::neverused()),
                    info_check: Arc::new(Mutex::new(info::Info::neverused())),
                    out: Arc::new(move |_, i| compiled.check(&mut i.duplicate(), None)),
                    run: Arc::new(move |_, i| compiled2.run(&mut i.duplicate())),
                },
            }),
        }))
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src
    }
}
