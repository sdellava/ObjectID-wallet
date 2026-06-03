use crate::{persistence::STRONGHOLD, state::credentials::VerifiableCredentialRecord};

use blake2::{Blake2b512, Digest};
use iota_stronghold::{
    procedures::{
        BIP39Generate, Curve, GenerateKey, KeyType, MnemonicLanguage, PublicKey, Slip10Derive, Slip10DeriveInput,
        StrongholdProcedure, WriteVault,
    },
    Client, KeyProvider, Location, SnapshotPath, Stronghold,
};
use log::info;
use stronghold_ext::{
    execute_procedure_ext,
    procs::{self, es256::Es256Procs},
};
use uuid::Uuid;
use zeroize::Zeroizing;

// This file is where we implement the stronghold library for our app, which is used to store sensitive data.
// We have to follow the hard-coded values used in `identity.rs` to make our Stronghold compatible.
static STRONGHOLD_VAULT_PATH: &str = "iota_identity_vault";
static STRONGHOLD_CLIENT_PATH: &[u8] = b"iota_identity_client";
static IOTA_WALLET_SEED_KEY: &str = "iota-wallet-seed";
static IOTA_WALLET_KEY: &str = "iota-wallet-ed25519-0";

/// This struct is the main point of communication between our appstate and the stronghold library.
#[derive(Debug)]
pub struct StrongholdManager {
    stronghold: Stronghold,
    client: Client,
    key_provider: KeyProvider,
    snapshot_path: SnapshotPath,
}

impl StrongholdManager {
    pub fn create(password: &str) -> anyhow::Result<Self> {
        let stronghold = Stronghold::default();
        let client_path = STRONGHOLD
            .lock()
            .unwrap()
            .to_str()
            .ok_or(anyhow::anyhow!("failed to get stronghold path"))?
            .to_owned();

        let snapshot_path = SnapshotPath::from_path(client_path.clone());
        let key_provider =
            KeyProvider::with_passphrase_hashed_blake2b(password.as_bytes().to_vec()).expect("failed to load key");

        let client: Client = stronghold
            .create_client(STRONGHOLD_CLIENT_PATH)
            .expect("cannot create client");

        // Generate ed25519 key
        {
            let ed25519_output_location = Location::generic(
                STRONGHOLD_VAULT_PATH.as_bytes().to_vec(),
                "ed25519-0".to_string().as_bytes().to_vec(),
            );

            info!("ed25519_output_location: {ed25519_output_location:?}");

            client
                .execute_procedure(StrongholdProcedure::GenerateKey(GenerateKey {
                    ty: KeyType::Ed25519,
                    output: ed25519_output_location,
                }))
                .expect("failed to generate new private key");

            info!("Successfully generated new private key with type Ed25519");
        }

        // Generate ES256 Key
        {
            let es256_output_location = Location::generic(
                STRONGHOLD_VAULT_PATH.as_bytes().to_vec(),
                "es256-0".to_string().as_bytes().to_vec(),
            );

            info!("es256_output_location: {es256_output_location:?}");

            execute_procedure_ext(
                &client,
                Es256Procs::GenerateKey(procs::es256::GenerateKey {
                    output: es256_output_location,
                }),
            )
            .expect("failed to generate new private key");

            info!("Successfully generated new private key with type Es256");
        }

        let stronghold_manager = Self {
            stronghold,
            client,
            key_provider,
            snapshot_path: snapshot_path.clone(),
        };

        stronghold_manager.commit()?;
        Ok(stronghold_manager)
    }

    pub fn load(password: &str) -> anyhow::Result<Self> {
        let stronghold = Stronghold::default();
        let client_path = STRONGHOLD
            .lock()
            .unwrap()
            .to_str()
            .ok_or(anyhow::anyhow!("failed to get stronghold path"))?
            .to_owned();
        let snapshot_path = SnapshotPath::from_path(client_path.clone());
        let key_provider =
            KeyProvider::with_passphrase_hashed_blake2b(password.as_bytes().to_vec()).expect("failed to load key");

        info!("Loading snapshot");

        let client = stronghold.load_client_from_snapshot(STRONGHOLD_CLIENT_PATH, &key_provider, &snapshot_path)?;

        Ok(Self {
            stronghold,
            client,
            key_provider,
            snapshot_path,
        })
    }

    pub fn commit(&self) -> anyhow::Result<()> {
        // Set the work factor to 10 to speed up the commit.
        // TODO: security: weaker encryption?
        engine::snapshot::try_set_encrypt_work_factor(10)?;

        self.stronghold
            .write_client(STRONGHOLD_CLIENT_PATH)
            .expect("store client state into snapshot state failed");

        self.stronghold
            .commit_with_keyprovider(&self.snapshot_path, &self.key_provider)
            .expect("stronghold could not commit");

        Ok(())
    }

    pub fn get(&self, key: Uuid) -> anyhow::Result<Option<Vec<u8>>> {
        let key = key.to_string().as_bytes().to_vec();
        let value = self.client.store().get(&key)?;

        Ok(value)
    }

    pub fn insert(&self, key: Uuid, value: Vec<u8>) -> anyhow::Result<()> {
        self.client
            .store()
            .insert(key.to_string().as_bytes().to_vec(), value, None)?;

        self.commit()
    }

    pub fn initialize_iota_wallet(&self) -> anyhow::Result<(String, String)> {
        let seed_location = stronghold_location(IOTA_WALLET_SEED_KEY);
        let key_location = stronghold_location(IOTA_WALLET_KEY);

        let mnemonic = self
            .client
            .execute_procedure(StrongholdProcedure::BIP39Generate(BIP39Generate {
                passphrase: Default::default(),
                language: MnemonicLanguage::English,
                output: seed_location.clone(),
            }))?
            .try_into()?;

        self.derive_iota_wallet_key(seed_location, key_location.clone())?;
        let address = self.iota_address_from_key(key_location)?;
        self.commit()?;

        Ok((mnemonic, address))
    }

    pub fn import_iota_seed(&self, seed: &str) -> anyhow::Result<String> {
        let seed_location = stronghold_location(IOTA_WALLET_SEED_KEY);
        let key_location = stronghold_location(IOTA_WALLET_KEY);

        self.client
            .execute_procedure(StrongholdProcedure::WriteVault(WriteVault {
                data: Zeroizing::new(seed.as_bytes().to_vec()),
                location: seed_location.clone(),
            }))?;

        self.derive_iota_wallet_key(seed_location, key_location.clone())?;
        let address = self.iota_address_from_key(key_location)?;
        self.commit()?;

        Ok(address)
    }

    fn derive_iota_wallet_key(&self, seed_location: Location, key_location: Location) -> anyhow::Result<()> {
        self.client
            .execute_procedure(StrongholdProcedure::Slip10Derive(Slip10Derive {
                curve: Curve::Ed25519,
                chain: vec![44, 4218, 0, 0, 0],
                input: Slip10DeriveInput::Seed(seed_location),
                output: key_location,
            }))?;

        Ok(())
    }

    fn iota_address_from_key(&self, key_location: Location) -> anyhow::Result<String> {
        let public_key: Vec<u8> = self
            .client
            .execute_procedure(StrongholdProcedure::PublicKey(PublicKey {
                ty: KeyType::Ed25519,
                private_key: key_location,
            }))?
            .into();

        let mut hasher = Blake2b512::new();
        hasher.update([0x00]);
        hasher.update(public_key);
        let digest = hasher.finalize();

        Ok(format!("0x{}", hex_lower(&digest[..32])))
    }

    // TODO: fix this function's return type.
    pub fn values(&self) -> anyhow::Result<Option<Vec<VerifiableCredentialRecord>>> {
        let client = self.client.clone();

        let mut keys = self.client.store().keys()?;
        keys.sort();
        keys.iter()
            .map(|key| {
                client
                    .store()
                    .get(key)
                    .map(|value| value.map(|value| serde_json::from_slice(&value)))
            })
            .collect::<Result<Option<serde_json::Result<_>>, _>>()?
            .transpose()
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub fn remove(&self, key: Uuid) -> anyhow::Result<Option<Vec<u8>>> {
        let value = self.client.store().delete(key.to_string().as_bytes())?;
        self.commit()?;

        Ok(value)
    }
}

fn stronghold_location(key: &str) -> Location {
    Location::generic(STRONGHOLD_VAULT_PATH.as_bytes().to_vec(), key.as_bytes().to_vec())
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    #[serial_test::serial]
    fn test_stronghold_manager() {
        let path = NamedTempFile::new().unwrap().into_temp_path();
        *STRONGHOLD.lock().unwrap() = path.as_os_str().into();

        let stronghold_manager = StrongholdManager::create("sup3rSecr3t").unwrap();

        let key = Uuid::new_v4();
        let value = "test".as_bytes().to_vec();

        stronghold_manager.insert(key, value).unwrap();

        let value = stronghold_manager.get(key).unwrap().unwrap();
        assert_eq!(value, "test".as_bytes().to_vec());

        stronghold_manager.remove(key).unwrap();

        let value = stronghold_manager.get(key).unwrap();
        assert!(value.is_none());
    }

    #[test]
    #[serial_test::serial]
    fn test_iota_wallet_initialization_and_import() {
        let path = NamedTempFile::new().unwrap().into_temp_path();
        *STRONGHOLD.lock().unwrap() = path.as_os_str().into();

        let stronghold_manager = StrongholdManager::create("sup3rSecr3t").unwrap();
        let (mnemonic, address) = stronghold_manager.initialize_iota_wallet().unwrap();

        assert!(mnemonic.split_whitespace().count() >= 12);
        assert!(address.starts_with("0x"));
        assert_eq!(address.len(), 66);

        let imported_address = stronghold_manager.import_iota_seed("objectid imported seed").unwrap();
        assert!(imported_address.starts_with("0x"));
        assert_eq!(imported_address.len(), 66);
    }
}
