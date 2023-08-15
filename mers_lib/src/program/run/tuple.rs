use std::sync::Arc;

use crate::{
    data::{self, tuple::TupleT, Data, Type},
    parsing::SourcePos,
};

use super::{CheckError, MersStatement};

#[derive(Debug)]
pub struct Tuple {
    pub pos_in_src: SourcePos,
    pub elems: Vec<Box<dyn MersStatement>>,
}
impl MersStatement for Tuple {
    fn check_custom(
        &self,
        info: &mut super::CheckInfo,
        init_to: Option<&Type>,
    ) -> Result<data::Type, super::CheckError> {
        let mut it = if let Some(init_to) = init_to {
            let mut vec = (0..self.elems.len())
                .map(|_| Type::empty())
                .collect::<Vec<_>>();
            for t in init_to.types.iter() {
                if let Some(t) = t.as_any().downcast_ref::<TupleT>() {
                    if t.0.len() == self.elems.len() {
                        for (i, e) in t.0.iter().enumerate() {
                            vec[i].add(Arc::new(e.clone()));
                        }
                    } else {
                        return Err(CheckError(
                            format!("can't init to statement type Tuple with value type {t}, which is part of {init_to} - only tuples with the same length ({}) can be assigned to tuples", self.elems.len()),
                        ));
                    }
                } else {
                    return Err(CheckError(
                        format!("can't init to statement type Tuple with value type {t}, which is part of {init_to} - only tuples can be assigned to tuples"),
                    ));
                }
            }
            Some(vec)
        } else {
            None
        };
        Ok(Type::new(data::tuple::TupleT(
            self.elems
                .iter()
                .map(|v| {
                    v.check(
                        info,
                        if let Some(it) = &mut it {
                            Some(it.pop().unwrap())
                        } else {
                            None
                        }
                        .as_ref(),
                    )
                })
                .collect::<Result<_, _>>()?,
        )))
    }
    fn run_custom(&self, info: &mut super::Info) -> crate::data::Data {
        Data::new(data::tuple::Tuple(
            self.elems.iter().map(|s| s.run(info)).collect(),
        ))
    }
    fn has_scope(&self) -> bool {
        false
    }
    fn pos_in_src(&self) -> &SourcePos {
        &self.pos_in_src
    }
}
