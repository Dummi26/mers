use std::sync::Arc;

use crate::{
    data::{self, Data, Type},
    errors::{CheckError, SourceRange},
};

use super::{Info, MersStatement};

#[derive(Debug)]
pub struct Loop {
    pub pos_in_src: SourceRange,
    pub inner: Box<dyn MersStatement>,
}

impl MersStatement for Loop {
    fn check_custom(
        &self,
        info: &mut super::CheckInfo,
        init_to: Option<&Type>,
    ) -> Result<data::Type, CheckError> {
        if init_to.is_some() {
            return Err("can't init to statement type If".to_string().into());
        }
        let mut t = Type::empty();
        let inner_return_type = self.inner.check(info, None)?;
        for i in inner_return_type.types.iter() {
            if let Some(i) = i.as_any().downcast_ref::<data::tuple::TupleT>() {
                if i.0.len() > 1 {
                    return Err(format!(
                        "Loop: Inner statement must return ()/(T), not {t} (because of {i}, a tuple of length > 1)."
                    )
                    .into());
                } else {
                    if let Some(i) = i.0.first() {
                        for i in i.types.iter() {
                            t.add(Arc::clone(i));
                        }
                    }
                }
            } else {
                return Err(format!(
                    "Loop: Inner statement must return ()/(T), not {t} (because of {i}, which isn't a tuple)."
                )
                .into());
            }
        }
        Ok(t)
    }
    fn run_custom(&self, info: &mut Info) -> Data {
        loop {
            if let Some(v) = self.inner.run(info).one_tuple_content() {
                return v;
            }
        }
    }
    fn has_scope(&self) -> bool {
        true
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src.clone()
    }
    fn inner_statements(&self) -> Vec<&dyn MersStatement> {
        vec![self.inner.as_ref()]
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
