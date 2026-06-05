use crate::state::actions::ActionTrait;
use crate::state::iota_wallet::{reducers::import_seed_identity::import_seed_identity, IotaNetwork};
use crate::{reducer, state::Reducer};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Debug, TS, Clone)]
#[ts(export, export_to = "bindings/actions/ImportIotaSeedIdentity.ts")]
pub struct ImportIotaSeedIdentity {
    pub seed_hex: String,
    pub did: String,
    #[serde(default)]
    pub network: IotaNetwork,
    #[serde(default)]
    pub expected_address: Option<String>,
}

#[typetag::serde(name = "[IOTA Wallet] Import seed identity")]
impl ActionTrait for ImportIotaSeedIdentity {
    fn reducers<'a>(&self) -> Vec<Reducer<'a>> {
        vec![reducer!(import_seed_identity)]
    }
}
