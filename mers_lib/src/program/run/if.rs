use super::MersStatement;

#[derive(Debug)]
pub struct If {
    pub condition: Box<dyn MersStatement>,
    pub on_true: Box<dyn MersStatement>,
    pub on_false: Option<Box<dyn MersStatement>>,
}

impl MersStatement for If {
    fn run_custom(&self, info: &mut super::Info) -> crate::data::Data {
        self.condition.run(info);
        todo!("what now?")
    }
    fn has_scope(&self) -> bool {
        true
    }
}
