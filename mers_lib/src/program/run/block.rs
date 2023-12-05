use crate::{
    data::Type,
    errors::{CheckError, SourceRange},
};

use super::{CheckInfo, MersStatement};

#[derive(Debug)]
pub struct Block {
    pub pos_in_src: SourceRange,
    pub statements: Vec<Box<dyn MersStatement>>,
}
impl MersStatement for Block {
    fn check_custom(
        &self,
        info: &mut CheckInfo,
        init_to: Option<&Type>,
    ) -> Result<crate::data::Type, CheckError> {
        if init_to.is_some() {
            return Err("can't init to statement type Block".to_string().into());
        }
        let mut o = Type::empty_tuple();
        for s in &self.statements {
            o = s.check(info, None)?;
        }
        Ok(o)
    }
    fn run_custom(&self, info: &mut super::Info) -> crate::data::Data {
        self.statements
            .iter()
            .map(|s| s.run(info))
            .last()
            .unwrap_or_else(|| crate::data::Data::new(crate::data::tuple::Tuple(vec![])))
    }
    fn has_scope(&self) -> bool {
        true
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src.clone()
    }
    fn inner_statements(&self) -> Vec<&dyn MersStatement> {
        self.statements.iter().map(|s| s.as_ref()).collect()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
