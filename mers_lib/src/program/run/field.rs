use crate::{
    data::{self, object::ObjectT, Data, MersDataWInfo, MersTypeWInfo, Type},
    errors::{CheckError, EColor, SourceRange},
};

use super::MersStatement;

#[derive(Debug)]
pub struct Field {
    pub pos_in_src: SourceRange,
    pub object: Box<dyn MersStatement>,
    pub field_str: String,
    pub field: usize,
}
impl MersStatement for Field {
    fn check_custom(
        &self,
        info: &mut super::CheckInfo,
        init_to: Option<&Type>,
    ) -> Result<data::Type, super::CheckError> {
        if init_to.is_some() {
            return Err("can't init to statement type Field".to_string().into());
        }
        let object = self.object.check(info, init_to)?;
        let mut o = Type::empty();
        for t in object.types.iter() {
            if let Some(t) = t.as_any().downcast_ref::<ObjectT>() {
                if let Some(t) = t.get(self.field) {
                    o.add_all(t);
                } else {
                    return Err(CheckError::new().msg(vec![
                        ("can't get field ".to_owned(), None),
                        (self.field_str.clone(), Some(EColor::ObjectField)),
                        (" of object ".to_owned(), None),
                        (t.with_info(info).to_string(), Some(EColor::InitFrom)),
                    ]));
                }
            } else {
                return Err(CheckError::new().msg(vec![
                    ("can't get field ".to_owned(), None),
                    (self.field_str.clone(), Some(EColor::ObjectField)),
                    (" of non-object type ".to_owned(), None),
                    (t.with_info(info).to_string(), Some(EColor::InitFrom)),
                ]));
            }
        }
        Ok(o)
    }
    fn run_custom(&self, info: &mut super::Info) -> Result<Data, CheckError> {
        let object = self.object.run(info)?;
        let object = object.get();
        let object = object
            .as_any()
            .downcast_ref::<data::object::Object>()
            .ok_or_else(|| {
                format!(
                    "couldn't extract field {} from non-object value {}",
                    self.field_str,
                    object.with_info(info)
                )
            })?;
        Ok(object
            .get(self.field)
            .ok_or_else(|| {
                format!(
                    "couldn't extract field {} from object {}",
                    self.field_str,
                    object.with_info(info)
                )
            })?
            .clone())
    }
    fn has_scope(&self) -> bool {
        false
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src.clone()
    }
    fn inner_statements(&self) -> Vec<&dyn MersStatement> {
        vec![&*self.object]
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
