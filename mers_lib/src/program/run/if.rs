use std::sync::Arc;

use crate::{
    data::{self, tuple::TupleT, Data, Type},
    errors::{CheckError, EColor, SourceRange},
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
        if !cond_return_type.is_included_in(&data::bool::bool_type()) {
            return Err(CheckError::new()
                .src(vec![
                    (self.pos_in_src.clone(), None),
                    (
                        self.condition.source_range(),
                        Some(EColor::IfConditionNotBool),
                    ),
                ])
                .msg(vec![
                    ("The ".to_owned(), None),
                    ("condition".to_owned(), Some(EColor::IfConditionNotBool)),
                    (
                        " in an if-statement must return Bool, not ".to_owned(),
                        None,
                    ),
                    (
                        cond_return_type.simplified_as_string(info),
                        Some(EColor::IfConditionNotBool),
                    ),
                ]));
        }
        let may_be_true = Type::new(data::bool::TrueT).is_included_in(&cond_return_type);
        let may_be_false = Type::new(data::bool::FalseT).is_included_in(&cond_return_type);
        let mut t = if may_be_true {
            self.on_true.check(info, None)?
        } else {
            Type::empty()
        };
        if may_be_false {
            if let Some(f) = &self.on_false {
                t.add_all(&f.check(info, None)?);
            } else {
                t.add(Arc::new(TupleT(vec![])));
            }
        }
        if let Some(show_warning) = &info.global.show_warnings {
            if !may_be_false || !may_be_true {
                let mut e = CheckError::new().src(vec![
                    (self.pos_in_src.clone(), None),
                    (
                        self.condition.source_range(),
                        Some(EColor::IfConditionNotBool),
                    ),
                ]);
                if !may_be_true {
                    e.msg_mut(vec![(
                        "Condition in this if-statement is never true".to_owned(),
                        Some(EColor::Warning),
                    )]);
                }
                if !may_be_false {
                    e.msg_mut(vec![(
                        "Condition in this if-statement is never false".to_owned(),
                        Some(EColor::Warning),
                    )]);
                }
                show_warning(e);
            }
        }
        Ok(t)
    }
    fn run_custom(&self, info: &mut super::Info) -> Result<Data, CheckError> {
        Ok(
            if let Some(data::bool::Bool(true)) = self
                .condition
                .run(info)?
                .get()
                .as_any()
                .downcast_ref::<data::bool::Bool>()
            {
                self.on_true.run(info)?
            } else if let Some(on_false) = &self.on_false {
                on_false.run(info)?
            } else {
                Data::empty_tuple()
            },
        )
    }
    fn has_scope(&self) -> bool {
        true
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
