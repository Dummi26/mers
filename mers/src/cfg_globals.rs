use mers_lib::{
    data::{self, Data},
    prelude_extend_config::*,
};

pub fn add_general(cfg: Config) -> Config {
    cfg.add_var("mers_cli".to_string(), Data::new(data::bool::Bool(true)))
}
