use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Module {
    pub id: String
}


#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MainConfig {
    pub module: Module,
}