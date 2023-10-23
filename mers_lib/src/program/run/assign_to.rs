use colored::Colorize;

use crate::data::{self, Data, MersType, Type};

use super::{CheckError, CheckInfo, MersStatement, SourceRange};

#[derive(Debug)]
pub struct AssignTo {
    pub pos_in_src: SourceRange,
    pub is_init: bool,
    pub target: Box<dyn MersStatement>,
    pub source: Box<dyn MersStatement>,
}

impl MersStatement for AssignTo {
    fn check_custom(
        &self,
        info: &mut CheckInfo,
        init_to: Option<&Type>,
    ) -> Result<Type, CheckError> {
        if init_to.is_some() {
            return Err("can't init to statement type AssignTo".to_string().into());
        }
        let source = self.source.check(info, None)?;
        let target = match self.target.check(info, Some(&source)) {
            Ok(v) => v,
            Err(e) => {
                return Err(CheckError::new()
                    .src(vec![
                        (self.pos_in_src, None),
                        (
                            self.target.source_range(),
                            Some(colored::Color::BrightYellow),
                        ),
                        (self.source.source_range(), Some(colored::Color::BrightCyan)),
                    ])
                    .msg(format!("Cannot initialize:"))
                    .err(e))
            }
        };
        if !self.is_init {
            if let Some(t) = target.dereference() {
                if !source.is_included_in(&t) {
                    return Err(CheckError::new()
                        .src(vec![
                            (self.pos_in_src, None),
                            (
                                self.target.source_range(),
                                Some(colored::Color::BrightYellow),
                            ),
                            (self.source.source_range(), Some(colored::Color::BrightCyan)),
                        ])
                        .msg(format!(
                            "can't assign {} to {} because it isn't included in {}",
                            source.to_string().bright_cyan(),
                            target.to_string().bright_yellow(),
                            t
                        )));
                }
            } else {
                return Err(CheckError::new()
                    .src(vec![
                        (self.pos_in_src, None),
                        (self.target.source_range(), Some(colored::Color::Red)),
                    ])
                    .msg(format!("can't assign to non-reference!")));
            }
        }
        Ok(Type::empty_tuple())
    }
    fn run_custom(&self, info: &mut super::Info) -> crate::data::Data {
        let source = self.source.run(info);
        let target = self.target.run(info);
        data::defs::assign(&source, &target);
        Data::empty_tuple()
    }
    fn has_scope(&self) -> bool {
        false
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src
    }
}
