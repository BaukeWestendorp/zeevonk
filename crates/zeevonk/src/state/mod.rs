use crate::state::patch::Patch;

pub mod fixture;
pub mod patch;

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct State {
    pub(crate) patch: Patch,
}

impl State {
    pub fn patch(&self) -> &Patch {
        &self.patch
    }
}
