use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Record {
    pub(crate) owner: String,
    pub(crate) title: String,
    pub(crate) amount: f32,
    pub(crate) tags: Vec<String>,
}
