use crate::error::FaucetError;
use crate::wallet::history::{TransactionHistory, TransactionRecord};
use std::path::PathBuf;
use tracing::info;
use zingolib::{
    lightclient::LightClient,
    config::ZingoConfig,
};
// Import from axum instead of separate http crate
use axum::http::Uri;

#[derive(Debug, Clone)]
pub struct Balance {
    pub transparent: u64,
    pub sapling: u64,
    pub orchard: u64,
}

impl Balance {
    pub fn total_zatoshis(&self) -> u64 {
        self.transparent + self.sapling + self.orchard
    }

    pub fn total_zec(&self) -> f64 {
        self.total_zatoshis() as f64 / 100_000_000.0
    }

    pub fn orchard_zec(&self) -> f64 {
        self.orchard as f64 / 100_000_000.0
    }

    pub fn transparent_zec(&self) -> f64 {
        self.transparent as f64 / 100_000_000.0
    }
}

pub struct WalletManager {
    client: LightClient,
    history: TransactionHistory,
}

impl WalletManager {
    pub async fn new(
        data_dir: PathBuf,
        server_uri: String,
    ) -> Result<Self, FaucetError> {
        info!("Initializing ZingoLib LightClient");
        
        // Parse the server URI using axum::http::Uri
        let uri: Uri = server_uri.parse().map_err(|e| {
            FaucetError::Wallet(format!("Invalid server URI: {}", e))
        })?;

        // Create wallet directory if it doesn't exist
        std::fs::create_dir_all(&data_dir).map_err(|e| {
            FaucetError::Wallet(format!("Failed to create wallet directory: {}", e))
        })?;

        // Build configuration for regtest
        let config = ZingoConfig::build(zingolib::config::ChainType::Regtest(Default::default()))
            .set_lightwalletd_uri(uri)  // Pass Uri directly, not wrapped in Arc<RwLock>
            .set_wallet_dir(data_dir.clone())
            .create();

        // Try to load existing wallet or create new one
        let wallet_path = data_dir.join("zingo-wallet.dat");
        let client = if wallet_path.exists() {
            info!("Loading existing wallet from {:?}", wallet_path);
            LightClient::create_from_wallet_path(config).map_err(|e| {
                FaucetError::Wallet(format!("Failed to load wallet: {}", e))
            })?
        } else {
            info!("Creating new wallet");
            LightClient::new(
                config,
                zcash_primitives::consensus::BlockHeight::from_u32(0),
                false,
            ).map_err(|e| {
                FaucetError::Wallet(format!("Failed to create wallet: {}", e))
            })?
        };

        // Initialize transaction history
        let history = TransactionHistory::load(&data_dir)?;

        // Sync wallet
        info!("Syncing wallet with chain...");
        let mut client_mut = client;
        client_mut.sync().await.map_err(|e| {
            FaucetError::Wallet(format!("Sync failed: {}", e))
        })?;

        info!("Wallet initialized successfully");

        Ok(Self { client: client_mut, history })
    }

    pub async fn get_unified_address(&self) -> Result<String, FaucetError> {
        // TODO: Update this to match actual zingolib API
        // The method names and return types need to be verified against your zingolib version
        let _wallet = self.client.wallet.read().await;
        
        // This is a placeholder - you need to check the actual API
        // Possible methods: wallet.addresses(), wallet.get_all_addresses(), etc.
        Err(FaucetError::Wallet(
            "get_unified_address() needs implementation for this zingolib version".to_string()
        ))
    }

    pub async fn get_balance(&self) -> Result<Balance, FaucetError> {
        // TODO: Update this to match actual zingolib API
        let _wallet = self.client.wallet.read().await;
        
        // This is a placeholder - you need to check the actual API
        // The balance calculation method will depend on your zingolib version
        Err(FaucetError::Wallet(
            "get_balance() needs implementation for this zingolib version".to_string()
        ))
    }

    pub async fn send_transaction(
        &mut self,
        to_address: &str,
        amount_zec: f64,
        _memo: Option<String>,
    ) -> Result<String, FaucetError> {
        info!("Sending {} ZEC to {}", amount_zec, &to_address[..to_address.len().min(16)]);

        // Convert ZEC to zatoshis
        let amount_zatoshis = (amount_zec * 100_000_000.0) as u64;

        // Check balance
        let balance = self.get_balance().await?;
        if balance.orchard < amount_zatoshis {
            return Err(FaucetError::InsufficientBalance(format!(
                "Need {} ZEC, have {} ZEC in Orchard pool",
                amount_zec,
                balance.orchard_zec()
            )));
        }

        // TODO: Update this to match actual zingolib API
        // The send method signature will depend on your zingolib version
        Err(FaucetError::TransactionFailed(
            "send_transaction() needs implementation for this zingolib version. \
             Please check zingolib documentation for the correct API.".to_string()
        ))
    }

    pub async fn sync(&mut self) -> Result<(), FaucetError> {
        self.client.sync().await.map_err(|e| {
            FaucetError::Wallet(format!("Sync failed: {}", e))
        })?;
        Ok(())
    }

    pub fn get_transaction_history(&self, limit: usize) -> Vec<TransactionRecord> {
        self.history.get_recent(limit)
    }

    pub fn get_stats(&self) -> (usize, f64) {
        let txs = self.history.get_all();
        let count = txs.len();
        let total_sent: f64 = txs.iter().map(|tx| tx.amount).sum();
        (count, total_sent)
    }
}