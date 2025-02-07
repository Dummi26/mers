use crate::{
    data::object::ObjectFieldsMap,
    errors::{CheckError, SourceRange},
    info, program,
};

use super::{CompInfo, MersStatement};

/// Extracts a property with the given name from the object
#[derive(Debug)]
pub struct Field {
    pub pos_in_src: SourceRange,
    pub object: Box<dyn MersStatement>,
    pub field: String,
}
impl MersStatement for Field {
    fn has_scope(&self) -> bool {
        false
    }
    fn compile_custom(
        &self,
        info: &mut info::Info<super::Local>,
        comp: CompInfo,
    ) -> Result<Box<dyn program::run::MersStatement>, CheckError> {
        Ok(Box::new(program::run::field::Field {
            pos_in_src: self.pos_in_src.clone(),
            object: self.object.compile(info, comp)?,
            field_str: self.field.clone(),
            field: info.global.object_fields.get_or_add_field(&self.field),
        }))
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
