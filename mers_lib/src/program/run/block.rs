use crate::{data::Type, parsing::SourcePos};

use super::{CheckError, MersStatement};

#[derive(Debug)]
pub struct Block {
    pub pos_in_src: SourcePos,
    pub statements: Vec<Box<dyn MersStatement>>,
}
impl MersStatement for Block {
    fn check_custom(
        &self,
        info: &mut super::CheckInfo,
        init_to: Option<&Type>,
    ) -> Result<crate::data::Type, super::CheckError> {
        if init_to.is_some() {
            return Err(CheckError("can't init to statement type Block".to_string()));
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
    fn pos_in_src(&self) -> &SourcePos {
        &self.pos_in_src
    }
}
