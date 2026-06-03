use crate::{
    error::AppError,
    state::{
        actions::{listen, Action},
        iota_wallet::{IotaWalletState, StoredIotaWallet},
        AppState,
    },
};

use iota_keys::keystore::{AccountKeystore, InMemKeystore};
use iota_sdk::types::crypto::{EncodeDecodeBase64, SignatureScheme};

pub const IOTA_WALLET_STORE_KEY: &str = "iota-wallet";

pub async fn create_or_load_wallet(state: AppState, action: Action) -> Result<AppState, AppError> {
    let Some(payload) =
        listen::<crate::state::iota_wallet::actions::create_or_load_wallet::CreateOrLoadIotaWallet>(action)
    else {
        return Ok(state);
    };

    let wallet = create_or_load_wallet_for_setup(&state, payload.network, None).await?;

    Ok(AppState {
        iota_wallet: IotaWalletState::from(&wallet),
        ..state
    })
}

pub(crate) async fn create_or_load_wallet_for_setup(
    state: &AppState,
    network: crate::state::iota_wallet::IotaNetwork,
    imported_mnemonic: Option<String>,
) -> Result<StoredIotaWallet, AppError> {
    let stronghold_manager = state
        .core_utils
        .managers
        .lock()
        .await
        .stronghold_manager
        .clone()
        .ok_or(AppError::MissingManagerError("stronghold"))?;

    if imported_mnemonic.is_none() {
        if let Some(bytes) = stronghold_manager
            .get_named(IOTA_WALLET_STORE_KEY)
            .map_err(AppError::StrongholdValuesError)?
        {
            let mut wallet = serde_json::from_slice::<StoredIotaWallet>(&bytes).map_err(AppError::DeserializeFailed)?;
            if wallet.network != network {
                wallet.network = network;
                save_stored_wallet(state, &wallet).await?;
            }
            return Ok(wallet);
        }
    }

    let (address, mnemonic, public_key) = create_wallet_material(imported_mnemonic)?;
    let wallet = StoredIotaWallet {
        mnemonic,
        address,
        public_key: Some(public_key),
        identity_controller_private_key: None,
        identity_controller_public_jwk: None,
        network,
        did: None,
        did_network: None,
        did_document: None,
        identity_controller_cap: None,
    };
    let bytes = serde_json::to_vec(&wallet).map_err(AppError::DeserializeFailed)?;
    stronghold_manager
        .insert_named(IOTA_WALLET_STORE_KEY, bytes)
        .map_err(AppError::StrongholdInsertionError)?;

    Ok(wallet)
}

fn create_wallet_material(imported_mnemonic: Option<String>) -> Result<(String, String, String), AppError> {
    let mut keystore = InMemKeystore::default();
    let (address, mnemonic) = match imported_mnemonic {
        Some(mnemonic) => {
            let address = keystore
                .import_from_mnemonic(
                    &mnemonic,
                    SignatureScheme::ED25519,
                    None,
                    Some("objectid-iota".to_string()),
                )
                .map_err(|e| AppError::Error(format!("Failed to import IOTA wallet: {e}")))?;
            (address, mnemonic)
        }
        None => {
            let (address, mnemonic, _) = keystore
                .generate_and_add_new_key(
                    SignatureScheme::ED25519,
                    Some("objectid-iota".to_string()),
                    None,
                    Some("word24".to_string()),
                )
                .map_err(|e| AppError::Error(format!("Failed to create IOTA wallet: {e}")))?;
            (address, mnemonic)
        }
    };
    let public_key = keystore
        .get_key(&address)
        .map_err(|e| AppError::Error(format!("Failed to read generated IOTA public key: {e}")))?
        .public()
        .encode_base64();

    Ok((address.to_string(), mnemonic, public_key))
}

pub(crate) async fn load_stored_wallet(state: &AppState) -> Result<StoredIotaWallet, AppError> {
    let managers = state.core_utils.managers.lock().await;
    let stronghold_manager = managers
        .stronghold_manager
        .clone()
        .ok_or(AppError::MissingManagerError("stronghold"))?;
    drop(managers);

    let bytes = stronghold_manager
        .get_named(IOTA_WALLET_STORE_KEY)
        .map_err(AppError::StrongholdValuesError)?
        .ok_or_else(|| AppError::Error("IOTA wallet has not been initialized yet".to_string()))?;

    serde_json::from_slice::<StoredIotaWallet>(&bytes).map_err(AppError::DeserializeFailed)
}

pub(crate) async fn save_stored_wallet(state: &AppState, wallet: &StoredIotaWallet) -> Result<(), AppError> {
    let managers = state.core_utils.managers.lock().await;
    let stronghold_manager = managers
        .stronghold_manager
        .clone()
        .ok_or(AppError::MissingManagerError("stronghold"))?;
    drop(managers);

    let bytes = serde_json::to_vec(wallet).map_err(AppError::DeserializeFailed)?;
    stronghold_manager
        .insert_named(IOTA_WALLET_STORE_KEY, bytes)
        .map_err(AppError::StrongholdInsertionError)
}
