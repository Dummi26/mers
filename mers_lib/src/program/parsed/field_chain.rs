use crate::{
    data::object::ObjectFieldsMap,
    errors::{CheckError, SourceRange},
    info, program,
};

use super::{CompInfo, MersStatement};

/// Extracts a property with the given name from the object
#[derive(Debug)]
pub struct FieldChain {
    pub pos_in_src: SourceRange,
    pub object: Box<dyn MersStatement>,
    pub args: Option<(Vec<Box<dyn MersStatement>>, SourceRange)>,
    pub field: String,
    pub field_pos: SourceRange,
}
impl MersStatement for FieldChain {
    fn has_scope(&self) -> bool {
        false
    }
    fn compile_custom(
        &self,
        info: &mut info::Info<super::Local>,
        comp: CompInfo,
    ) -> Result<Box<dyn program::run::MersStatement>, CheckError> {
        Ok(Box::new(program::run::field_chain::FieldChain {
            pos_in_src: self.pos_in_src.clone(),
            object: self.object.compile(info, comp)?,
            args: if let Some((args, pos)) = &self.args {
                Some((
                    args.iter()
                        .map(|arg| arg.compile(info, comp))
                        .collect::<Result<Vec<_>, _>>()?,
                    pos.clone(),
                ))
            } else {
                None
            },
            field_str: self.field.clone(),
            field_pos: self.field_pos.clone(),
            field: info.global.object_fields.get_or_add_field(&self.field),
        }))
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src.clone()
    }
    fn inner_statements(&self) -> Vec<&dyn MersStatement> {
        let mut o = vec![&*self.object];
        if let Some((args, _)) = &self.args {
            o.extend(args.iter().map(|v| &**v));
        }
        o
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
