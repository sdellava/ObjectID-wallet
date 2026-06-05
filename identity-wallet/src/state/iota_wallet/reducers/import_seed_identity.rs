use crate::{
    error::AppError,
    state::{
        actions::{listen, Action},
        iota_wallet::{
            actions::import_seed_identity::ImportIotaSeedIdentity,
            reducers::create_identity::{ensure_identity_document_has_controller_and_key, identity_client_for_wallet},
            reducers::create_or_load_wallet::save_stored_wallet,
            IotaWalletState, StoredIotaWallet,
        },
        AppState,
    },
};

use identity_iota::iota::IotaDID;
use iota_keys::keystore::{AccountKeystore, InMemKeystore};
use iota_sdk::types::crypto::{EncodeDecodeBase64, SignatureScheme};

fn strip_0x(value: &str) -> &str {
    value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .unwrap_or(value)
}

fn decode_seed_hex(seed_hex: &str) -> Result<Vec<u8>, AppError> {
    let seed = strip_0x(seed_hex.trim());
    if !matches!(seed.len(), 64 | 128) || !seed.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(AppError::Error(
            "Imported seed must be 64 or 128 hex characters.".to_string(),
        ));
    }

    let mut bytes = Vec::with_capacity(seed.len() / 2);
    for i in (0..seed.len()).step_by(2) {
        let byte = u8::from_str_radix(&seed[i..i + 2], 16)
            .map_err(|e| AppError::Error(format!("Invalid imported seed hex: {e}")))?;
        bytes.push(byte);
    }
    Ok(bytes)
}

fn did_network_matches(did: &str, network: crate::state::iota_wallet::IotaNetwork) -> bool {
    match network {
        crate::state::iota_wallet::IotaNetwork::Testnet => did.starts_with("did:iota:testnet:"),
        crate::state::iota_wallet::IotaNetwork::Mainnet => {
            did.starts_with("did:iota:") && !did.starts_with("did:iota:testnet:")
        }
    }
}

pub async fn import_seed_identity(state: AppState, action: Action) -> Result<AppState, AppError> {
    let Some(payload) = listen::<ImportIotaSeedIdentity>(action) else {
        return Ok(state);
    };

    let did = payload.did.trim().to_string();
    if did.is_empty() {
        return Err(AppError::Error("Imported QR is missing the DID.".to_string()));
    }
    IotaDID::parse(&did).map_err(|e| AppError::Error(format!("Imported DID is invalid: {e}")))?;
    if !did_network_matches(&did, payload.network) {
        return Err(AppError::Error(
            "Imported DID network does not match the QR network.".to_string(),
        ));
    }

    let seed_bytes = decode_seed_hex(&payload.seed_hex)?;
    let seed_hex = strip_0x(payload.seed_hex.trim()).to_lowercase();

    let mut keystore = InMemKeystore::default();
    let address_obj = keystore
        .import_from_seed(
            &seed_bytes,
            SignatureScheme::ED25519,
            None,
            Some("objectid-iota".to_string()),
        )
        .map_err(|e| AppError::Error(format!("Failed to import IOTA seed: {e}")))?;
    let address = address_obj.to_string();

    if let Some(expected_address) = payload
        .expected_address
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        if !address.eq_ignore_ascii_case(expected_address) {
            return Err(AppError::Error(format!(
                "Imported seed address mismatch. QR={expected_address} derived={address}"
            )));
        }
    }

    let public_key = keystore
        .get_key(&address_obj)
        .map_err(|e| AppError::Error(format!("Failed to read imported IOTA public key: {e}")))?
        .public()
        .encode_base64();
    let key_pair = keystore
        .get_key(&address_obj)
        .and_then(|key| key.as_keypair())
        .map_err(|e| AppError::Error(format!("Failed to read imported IOTA key pair: {e}")))?;
    let private_key_hex = crate::state::iota_wallet::reducers::create_identity::hex_lower(&key_pair.to_bytes_no_flag());
    let public_jwk =
        crate::state::iota_wallet::reducers::create_identity::public_ed25519_jwk_json(key_pair.public().as_ref())?;

    let mut wallet = StoredIotaWallet {
        mnemonic: seed_hex,
        address,
        public_key: Some(public_key),
        identity_controller_private_key: Some(private_key_hex),
        identity_controller_public_jwk: Some(public_jwk),
        network: payload.network,
        did: Some(did),
        did_network: Some(payload.network),
        did_document: None,
        identity_controller_cap: None,
    };

    let identity_client = match identity_client_for_wallet(&wallet).await {
        Ok(identity_client) => identity_client,
        Err(e) => {
            let last_error = e.to_string();
            save_stored_wallet(&state, &wallet).await?;
            return Ok(AppState {
                iota_wallet: IotaWalletState {
                    last_error: Some(last_error),
                    ..IotaWalletState::from(&wallet)
                },
                ..state
            });
        }
    };

    let parsed_did = IotaDID::parse(wallet.did.as_deref().unwrap_or_default())
        .map_err(|e| AppError::Error(format!("Imported DID is invalid: {e}")))?;
    match identity_client.resolve_did(&parsed_did).await {
        Ok(mut document) => {
            ensure_identity_document_has_controller_and_key(&mut wallet, &mut document)?;
            wallet.did_document = serde_json::to_string_pretty(&document).ok();
        }
        Err(e) => {
            wallet.did_document = None;
            wallet.identity_controller_cap = None;
            save_stored_wallet(&state, &wallet).await?;
            return Ok(AppState {
                iota_wallet: IotaWalletState {
                    last_error: Some(format!("Imported wallet saved, but failed to resolve DID: {e}")),
                    ..IotaWalletState::from(&wallet)
                },
                ..state
            });
        }
    }

    save_stored_wallet(&state, &wallet).await?;

    Ok(AppState {
        iota_wallet: IotaWalletState::from(&wallet),
        ..state
    })
}
