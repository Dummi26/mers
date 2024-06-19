use colored::Colorize;

use crate::{
    data::{Data, Type},
    errors::{error_colors, CheckError, SourceRange},
    parsing::Source,
};

use super::MersStatement;

#[derive(Debug)]
pub struct Chain {
    pub pos_in_src: SourceRange,
    pub first: Box<dyn MersStatement>,
    pub chained: Box<dyn MersStatement>,
    pub as_part_of_include: Option<Source>,
}
impl MersStatement for Chain {
    fn check_custom(
        &self,
        info: &mut super::CheckInfo,
        init_to: Option<&Type>,
    ) -> Result<Type, CheckError> {
        if init_to.is_some() {
            return Err("can't init to statement type Chain".to_string().into());
        }
        let prev_enable_hooks = info.global.enable_hooks;
        if self.as_part_of_include.is_some() {
            info.global.enable_hooks = false;
        }
        let arg = self.first.check(info, None)?;
        let func = self.chained.check(info, None)?;
        info.global.enable_hooks = prev_enable_hooks;
        let mut o = Type::empty();
        for func in &func.types {
            if let Some(func) = func
                .as_any()
                .downcast_ref::<crate::data::function::FunctionT>()
            {
                match func.o(&arg) {
                    Ok(t) => o.add_all(&t),
                    Err(e) => {
                        return Err(if let Some(_) = &self.as_part_of_include {
                            CheckError::new()
                                .src(vec![(
                                    self.pos_in_src.clone(),
                                    Some(error_colors::HashIncludeErrorInIncludedFile),
                                )])
                                .msg(
                                    "Error in #include:"
                                        .color(error_colors::HashIncludeErrorInIncludedFile)
                                        .to_string(),
                                )
                                .err_with_diff_src(e)
                        } else {
                            CheckError::new()
                                .src(vec![
                                    (self.pos_in_src.clone(), None),
                                    (
                                        self.first.source_range(),
                                        Some(error_colors::FunctionArgument),
                                    ),
                                    (self.chained.source_range(), Some(error_colors::Function)),
                                ])
                                .msg(format!(
                                    "Can't call {} with an argument of type {}:",
                                    "this function".color(error_colors::Function),
                                    arg.to_string().color(error_colors::FunctionArgument)
                                ))
                                .err(e)
                        })
                    }
                }
            } else {
                return Err(CheckError::new()
                    .src(vec![
                        (self.pos_in_src.clone(), None),
                        (
                            self.chained.source_range(),
                            Some(error_colors::ChainWithNonFunction),
                        ),
                    ])
                    .msg(format!(
                        "cannot chain with a non-function ({})",
                        func.to_string().color(error_colors::ChainWithNonFunction)
                    )));
            }
        }
        Ok(o)
    }
    fn run_custom(&self, info: &mut super::Info) -> Result<Data, CheckError> {
        let f = self.first.run(info)?;
        let c = self.chained.run(info)?;
        let c = c.get();
        if let Some(func) = c.as_any().downcast_ref::<crate::data::function::Function>() {
            match func.run(f) {
                Ok(v) => Ok(v),
                Err(e) => Err(if let Some(_) = &self.as_part_of_include {
                    CheckError::new()
                        .src(vec![(
                            self.pos_in_src.clone(),
                            Some(error_colors::HashIncludeErrorInIncludedFile),
                        )])
                        .msg(
                            "Error in #include:"
                                .color(error_colors::HashIncludeErrorInIncludedFile)
                                .to_string(),
                        )
                        .err_with_diff_src(e)
                } else {
                    CheckError::new()
                        .src(vec![
                            (self.pos_in_src.clone(), None),
                            (
                                self.first.source_range(),
                                Some(error_colors::FunctionArgument),
                            ),
                            (self.chained.source_range(), Some(error_colors::Function)),
                        ])
                        .msg(format!(
                            "Error in {}:",
                            "this function".color(error_colors::Function)
                        ))
                        .err(e)
                }),
            }
        } else {
            todo!("err: not a function");
        }
    }
    fn has_scope(&self) -> bool {
        false
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src.clone()
    }
    fn inner_statements(&self) -> Vec<&dyn MersStatement> {
        vec![self.first.as_ref(), self.chained.as_ref()]
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
