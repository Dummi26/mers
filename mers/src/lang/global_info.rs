use std::{collections::HashMap, sync::Arc};

use crate::libs;

use super::{builtins, val_type::VType};

pub type GSInfo = Arc<GlobalScriptInfo>;

#[derive(Debug)]
pub struct GlobalScriptInfo {
    pub libs: Vec<libs::Lib>,
    pub lib_fns: HashMap<String, (usize, usize)>,

    pub enum_variants: HashMap<String, usize>,

    pub custom_type_names: HashMap<String, usize>,
    pub custom_types: Vec<VType>,
}

impl GlobalScriptInfo {
    pub fn to_arc(self) -> GSInfo {
        Arc::new(self)
    }
}

impl Default for GlobalScriptInfo {
    fn default() -> Self {
        Self {
            libs: vec![],
            lib_fns: HashMap::new(),
            enum_variants: Self::default_enum_variants(),
            custom_type_names: HashMap::new(),
            custom_types: vec![],
        }
    }
}
impl GlobalScriptInfo {
    pub fn default_enum_variants() -> HashMap<String, usize> {
        builtins::EVS
            .iter()
            .enumerate()
            .map(|(i, v)| (v.to_string(), i))
            .collect()
    }
}
