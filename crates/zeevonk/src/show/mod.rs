use crate::show::patch::Patch;

pub mod fixture;
pub mod patch;

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ShowData {
    pub(crate) patch: Patch,
}

impl ShowData {
    pub fn patch(&self) -> &Patch {
        &self.patch
    }
}
