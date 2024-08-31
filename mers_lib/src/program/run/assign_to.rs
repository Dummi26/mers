use crate::{
    data::{self, Data, Type},
    errors::{CheckError, EColor, SourceRange},
};

use super::{CheckInfo, MersStatement};

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
        let target = match self
            .target
            .check(info, if self.is_init { Some(&source) } else { None })
        {
            Ok(v) => v,
            Err(e) => {
                if !self.is_init {
                    return Err(e);
                }
                return Err(CheckError::new()
                    .src(vec![
                        (self.pos_in_src.clone(), None),
                        (self.target.source_range(), Some(EColor::InitTo)),
                        (self.source.source_range(), Some(EColor::InitFrom)),
                    ])
                    .msg_str(format!("Cannot initialize:"))
                    .err(e));
            }
        };
        if !self.is_init {
            if let Some(t) = target.dereference() {
                if !source.is_included_in(&t) {
                    return Err(CheckError::new()
                        .src(vec![
                            (self.pos_in_src.clone(), None),
                            (self.target.source_range(), Some(EColor::AssignTo)),
                            (self.source.source_range(), Some(EColor::AssignFrom)),
                        ])
                        .msg(vec![
                            ("can't assign ".to_owned(), None),
                            (source.simplified_as_string(info), Some(EColor::AssignFrom)),
                            (" to ".to_owned(), None),
                            (target.simplified_as_string(info), Some(EColor::AssignTo)),
                            (" because it isn't included in ".to_owned(), None),
                            (t.simplified_as_string(info), None),
                        ]));
                }
            } else {
                return Err(CheckError::new()
                    .src(vec![
                        (self.pos_in_src.clone(), None),
                        (
                            self.target.source_range(),
                            Some(EColor::AssignTargetNonReference),
                        ),
                    ])
                    .msg(vec![
                        ("can't assign to ".to_owned(), None),
                        (
                            "non-reference!".to_owned(),
                            Some(EColor::AssignTargetNonReference),
                        ),
                    ]));
            }
        }
        Ok(Type::empty_tuple())
    }
    fn run_custom(&self, info: &mut super::Info) -> Result<Data, CheckError> {
        let source = self.source.run(info)?;
        let target = self.target.run(info)?;
        data::defs::assign(&source, &target);
        Ok(Data::empty_tuple())
    }
    fn has_scope(&self) -> bool {
        false
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src.clone()
    }
    fn inner_statements(&self) -> Vec<&dyn MersStatement> {
        vec![self.target.as_ref(), self.source.as_ref()]
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
