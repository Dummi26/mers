use std::sync::Arc;

use crate::{
    data::{Data, Type},
    parsing::SourcePos,
};

use super::{CheckError, MersStatement};

#[derive(Debug)]
pub struct Chain {
    pub pos_in_src: SourcePos,
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
            return Err(CheckError("can't init to statement type Chain".to_string()));
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
                    Err(e) =>
                    return Err(CheckError(format!(
                        "cannot run this function with this argument (type: {arg}), because it would cause the following error:\n{e}"
                    ))),
                }
            } else {
                return Err(CheckError(format!(
                    "cannot chain with a non-function ({func})"
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
    fn pos_in_src(&self) -> &SourcePos {
        &self.pos_in_src
    }
}
