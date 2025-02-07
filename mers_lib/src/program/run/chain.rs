use crate::{
    data::{Data, Type},
    errors::{CheckError, EColor, SourceRange},
    parsing::Source,
};

use super::MersStatement;

#[derive(Debug)]
pub struct Chain {
    pub pos_in_src: SourceRange,
    pub first: Box<dyn MersStatement>,
    pub chained: Box<dyn MersStatement>,
    pub as_part_of_include: Option<Source>,
}
impl MersStatement for Chain {
    fn check_custom(
        &self,
        info: &mut super::CheckInfo,
        init_to: Option<&Type>,
    ) -> Result<Type, CheckError> {
        if init_to.is_some() {
            return Err("can't init to statement type Chain".to_string().into());
        }
        let prev_enable_hooks = info.global.enable_hooks;
        if self.as_part_of_include.is_some() {
            info.global.enable_hooks = false;
        }
        let arg = self.first.check(info, None)?;
        let func = self.chained.check(info, None)?;
        info.global.enable_hooks = prev_enable_hooks;
        check(
            &arg,
            &func,
            info,
            self.pos_in_src.clone(),
            self.first.source_range(),
            self.chained.source_range(),
            self.as_part_of_include.as_ref(),
        )
    }
    fn run_custom(&self, info: &mut super::Info) -> Result<Data, CheckError> {
        let f = self.first.run(info)?;
        let c = self.chained.run(info)?;
        run(
            f,
            c,
            info,
            self.pos_in_src.clone(),
            self.first.source_range(),
            self.chained.source_range(),
            self.as_part_of_include.as_ref(),
        )
    }
    fn has_scope(&self) -> bool {
        false
    }
    fn source_range(&self) -> SourceRange {
        self.pos_in_src.clone()
    }
    fn inner_statements(&self) -> Vec<&dyn MersStatement> {
        vec![self.first.as_ref(), self.chained.as_ref()]
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub fn check(
    arg: &Type,
    func: &Type,
    info: &mut super::CheckInfo,
    pos_in_src: SourceRange,
    arg_pos: SourceRange,
    func_pos: SourceRange,
    as_part_of_include: Option<&Source>,
) -> Result<Type, CheckError> {
    let mut o = Type::empty();
    for func in &func.types {
        if let Some(func) = func.executable() {
            match func.o(&arg) {
                Ok(t) => o.add_all(&t),
                Err(e) => {
                    return Err(if let Some(_) = as_part_of_include {
                        CheckError::new()
                            .src(vec![(
                                pos_in_src.clone(),
                                Some(EColor::HashIncludeErrorInIncludedFile),
                            )])
                            .msg(vec![(
                                "Error in #include:".to_owned(),
                                Some(EColor::HashIncludeErrorInIncludedFile),
                            )])
                            .err_with_diff_src(e)
                    } else {
                        CheckError::new()
                            .src(vec![
                                (pos_in_src.clone(), None),
                                (arg_pos, Some(EColor::FunctionArgument)),
                                (func_pos, Some(EColor::Function)),
                            ])
                            .msg(vec![
                                ("Can't call ".to_owned(), None),
                                ("this function".to_owned(), Some(EColor::Function)),
                                (" with an argument of type ".to_owned(), None),
                                (
                                    arg.simplified_as_string(info),
                                    Some(EColor::FunctionArgument),
                                ),
                                (":".to_owned(), None),
                            ])
                            .err(e)
                    })
                }
            }
        } else {
            return Err(CheckError::new()
                .src(vec![
                    (pos_in_src, None),
                    (func_pos, Some(EColor::ChainWithNonFunction)),
                ])
                .msg(vec![
                    ("cannot chain with a non-function (".to_owned(), None),
                    (
                        func.simplified_as_string(info),
                        Some(EColor::ChainWithNonFunction),
                    ),
                    (")".to_owned(), None),
                ]));
        }
    }
    Ok(o)
}

pub fn run(
    arg: Data,
    func: Data,
    info: &mut super::Info,
    pos_in_src: SourceRange,
    _arg_pos: SourceRange,
    func_pos: SourceRange,
    as_part_of_include: Option<&Source>,
) -> Result<Data, CheckError> {
    let func = func.get();
    match func.execute(arg, &info.global) {
        Some(Ok(v)) => Ok(v),
        Some(Err(e)) => Err(if let Some(_) = &as_part_of_include {
            CheckError::new().err_with_diff_src(e).src(vec![(
                pos_in_src.clone(),
                Some(EColor::StacktraceDescendHashInclude),
            )])
        } else {
            CheckError::new()
                .err(e)
                .src(vec![(pos_in_src.clone(), Some(EColor::StacktraceDescend))])
        }),
        None => Err(CheckError::new()
            .msg_str("tried to chain with non-function".to_owned())
            .src(vec![(func_pos, Some(EColor::ChainWithNonFunction))])),
    }
}
