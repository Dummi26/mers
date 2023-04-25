use std::{collections::HashMap, sync::Arc};

use crate::libs;

pub type GSInfo = Arc<GlobalScriptInfo>;
// pub type GSMInfo = Arc<Mutex<GlobalScriptInfo>>;
pub struct GlobalScriptInfo {
    pub libs: Vec<libs::Lib>,
    pub enums: HashMap<String, usize>,
}
impl GlobalScriptInfo {
    pub fn to_arc(self) -> GSInfo {
        Arc::new(self)
    }
}
