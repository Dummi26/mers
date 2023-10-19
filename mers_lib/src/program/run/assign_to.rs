use crate::{
    data::{self, Data, MersType, Type},
    parsing::SourcePos,
};

use super::{CheckError, CheckInfo, MersStatement};

#[derive(Debug)]
pub struct AssignTo {
    pub pos_in_src: SourcePos,
    pub is_init: bool,
    pub target: Box<dyn MersStatement>,
    pub source: Box<dyn MersStatement>,
}

impl MersStatement for AssignTo {
    fn check_custom(
        &self,
        info: &mut CheckInfo,
        init_to: Option<&Type>,
    ) -> Result<Type, CheckError> {
        if init_to.is_some() {
            return Err(CheckError(
                "can't init to statement type AssignTo".to_string(),
            ));
        }
        let source = self.source.check(info, None)?;
        let target = self.target.check(info, Some(&source))?;
        if !self.is_init {
            if let Some(t) = target.dereference() {
                if !source.is_included_in(&t) {
                    return Err(CheckError(format!(
                        "can't assign {source} to {target} because it isn't included in {t}!"
                    )));
                }
            } else {
                return Err(CheckError(format!("can't assign to non-reference!")));
            }
        }
        Ok(Type::empty_tuple())
    }
    fn run_custom(&self, info: &mut super::Info) -> crate::data::Data {
        let source = self.source.run(info);
        let target = self.target.run(info);
        data::defs::assign(&source, &target);
        Data::empty_tuple()
    }
    fn has_scope(&self) -> bool {
        false
    }
    fn pos_in_src(&self) -> &SourcePos {
        &self.pos_in_src
    }
}
