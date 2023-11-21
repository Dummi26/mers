use colored::Colorize;

use crate::{
    data::{Data, MersType, Type},
    errors::{error_colors, CheckError, SourceRange},
    parsing::types::ParsedType,
};

use super::MersStatement;

#[derive(Debug)]
pub struct AsType {
    pub pos_in_src: SourceRange,
    pub statement: Box<dyn MersStatement>,
    pub as_type: Vec<ParsedType>,
    pub type_pos_in_src: SourceRange,
    /// if false, only return an error if type doesn't fit, but don't expand type if it fits
    pub expand_type: bool,
}

impl MersStatement for AsType {
    fn check_custom(
        &self,
        info: &mut super::CheckInfo,
        init_to: Option<&Type>,
    ) -> Result<Type, CheckError> {
        if init_to.is_some() {
            return Err("can't init to statement type AsType (move type annotations from initialization to statement?)".to_string().into());
        }
        let return_type = self.statement.check(info, None)?;
        let as_type =
            crate::parsing::types::type_from_parsed(&self.as_type, info).map_err(|e| {
                CheckError::new()
                    .src(vec![(
                        self.type_pos_in_src,
                        Some(error_colors::BadTypeFromParsed),
                    )])
                    .err(e)
            })?;
        if !return_type.is_included_in(&as_type) {
            return Err(CheckError::new()
                .src(vec![
                    (self.pos_in_src, None),
                    (
                        self.type_pos_in_src,
                        Some(error_colors::AsTypeTypeAnnotation),
                    ),
                    (
                        self.statement.source_range(),
                        Some(error_colors::AsTypeStatementWithTooBroadType),
                    ),
                ])
                .msg(format!(
                    "Type must be included in {}, but the actual type {} isn't.",
                    as_type
                        .to_string()
                        .color(error_colors::AsTypeTypeAnnotation),
                    return_type
                        .to_string()
                        .color(error_colors::AsTypeStatementWithTooBroadType)
                )));
        }
        Ok(if self.expand_type {
            as_type.clone()
        } else {
            return_type
        })
    }
    fn run_custom(&self, info: &mut super::Info) -> Data {
        self.statement.run(info)
    }
    fn has_scope(&self) -> bool {
        false
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src
    }
}
