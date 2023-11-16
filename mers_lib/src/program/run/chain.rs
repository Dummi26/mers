use std::sync::Arc;

use colored::Colorize;

use crate::{
    data::{Data, Type},
    errors::{error_colors, CheckError, SourceRange},
};

use super::MersStatement;

#[derive(Debug)]
pub struct Chain {
    pub pos_in_src: SourceRange,
    pub first: Box<dyn MersStatement>,
    pub chained: Box<dyn MersStatement>,
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
        let arg = self.first.check(info, None)?;
        let func = self.chained.check(info, None)?;
        let mut o = Type::empty();
        for func in &func.types {
            if let Some(func) = func
                .as_any()
                .downcast_ref::<crate::data::function::FunctionT>()
            {
                match (func.0)(&arg) {
                    Ok(t) => o.add(Arc::new(t)),
                    Err(e) => {
                        return Err(CheckError::new()
                            .src(vec![
                                (self.pos_in_src, None),
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
                            .err(e))
                    }
                }
            } else {
                return Err(CheckError::new()
                    .src(vec![
                        (self.pos_in_src, None),
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
    fn run_custom(&self, info: &mut super::Info) -> Data {
        let f = self.first.run(info);
        let c = self.chained.run(info);
        let c = c.get();
        if let Some(func) = c.as_any().downcast_ref::<crate::data::function::Function>() {
            func.run(f)
        } else {
            todo!("err: not a function");
        }
    }
    fn has_scope(&self) -> bool {
        false
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src
    }
}
