use std::{fmt::Debug, sync::Arc};

use crate::{
    errors::{CheckError, SourceRange},
    parsing::types::{type_from_parsed, ParsedType},
};

use super::{CompInfo, Info, MersStatement};

pub struct CustomType {
    pub pos_in_src: SourceRange,
    pub name: String,
    pub source: Result<Vec<ParsedType>, Box<dyn MersStatement>>,
}
impl MersStatement for CustomType {
    fn compile_custom(
        &self,
        info: &mut Info,
        comp: CompInfo,
    ) -> Result<Box<dyn crate::program::run::MersStatement>, CheckError> {
        let src = match &self.source {
            Ok(p) => Ok(p.clone()),
            Err(s) => Err(s.compile(info, comp)?),
        };
        Ok(Box::new(crate::program::run::custom_type::CustomType {
            pos_in_src: self.pos_in_src.clone(),
            name: self.name.clone(),
            source: Box::new(move |ci| match &src {
                Ok(parsed) => Ok(Ok(Arc::new(type_from_parsed(parsed, ci)?))),
                Err(statement) => Ok(Ok(Arc::new(statement.check(&mut ci.clone(), None)?))),
            }),
        }))
    }
    fn has_scope(&self) -> bool {
        false
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src.clone()
    }
    fn inner_statements(&self) -> Vec<&dyn MersStatement> {
        if let Err(s) = &self.source {
            vec![s.as_ref()]
        } else {
            vec![]
        }
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Debug for CustomType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "type {} <...>", self.name)
    }
}
