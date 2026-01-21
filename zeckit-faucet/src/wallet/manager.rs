use crate::error::FaucetError;
use crate::wallet::history::{TransactionHistory, TransactionRecord};
use std::path::PathBuf;
use tracing::info;
use zingolib::{
    lightclient::LightClient,
    config::ZingoConfig,
};
use axum::http::Uri;
use zcash_primitives::consensus::BlockHeight;

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
        
        let uri: Uri = server_uri.parse().map_err(|e| {
            FaucetError::Wallet(format!("Invalid server URI: {}", e))
        })?;

        std::fs::create_dir_all(&data_dir).map_err(|e| {
            FaucetError::Wallet(format!("Failed to create wallet directory: {}", e))
        })?;

        // FIX 1: Create proper regtest config with activation heights
        let regtest_params = zingolib::config::RegtestNetwork {
            // Set all activation heights to 1 for regtest (all features active from start)
            sapling_activation_height: 1,
            blossom_activation_height: 1,
            heartwood_activation_height: 1,
            canopy_activation_height: 1,
            nu5_activation_height: 1,
            nu6_activation_height: 1,
            ..Default::default()
        };

        let config = ZingoConfig::build(zingolib::config::ChainType::Regtest(regtest_params))
            .set_lightwalletd_uri(uri)
            .set_wallet_dir(data_dir.clone())
            .create();

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
                BlockHeight::from_u32(0),
                false,
            ).map_err(|e| {
                FaucetError::Wallet(format!("Failed to create wallet: {}", e))
            })?
        };

        let history = TransactionHistory::load(&data_dir)?;

        info!("Syncing wallet with chain...");
        let mut client_mut = client;
        client_mut.sync().await.map_err(|e| {
            FaucetError::Wallet(format!("Sync failed: {}", e))
        })?;

        info!("Wallet initialized successfully");

        Ok(Self { client: client_mut, history })
    }

    // FIX 2: Implement get_unified_address using actual zingolib API
    pub async fn get_unified_address(&self) -> Result<String, FaucetError> {
        let wallet = self.client.wallet.read().await;
        
        // Get the first address from the wallet
        // zingolib typically stores addresses in a vector
        let addresses = wallet.wallet_capability()
            .addresses()
            .iter()
            .map(|addr| addr.encode(&self.client.config.chain))
            .collect::<Vec<_>>();
        
        addresses.first()
            .ok_or_else(|| FaucetError::Wallet("No addresses found in wallet".to_string()))
            .map(|s| s.to_string())
    }

    // FIX 3: Implement get_balance using actual zingolib API
    pub async fn get_balance(&self) -> Result<Balance, FaucetError> {
        let wallet = self.client.wallet.read().await;
        
        // Get balance from wallet
        let balance = wallet.balance();
        
        Ok(Balance {
            transparent: balance.transparent_balance.unwrap_or(0),
            sapling: balance.sapling_balance.unwrap_or(0),
            orchard: balance.orchard_balance.unwrap_or(0),
        })
    }

    // FIX 4: Implement send_transaction using actual zingolib API
    pub async fn send_transaction(
        &mut self,
        to_address: &str,
        amount_zec: f64,
        memo: Option<String>,
    ) -> Result<String, FaucetError> {
        info!("Sending {} ZEC to {}", amount_zec, &to_address[..to_address.len().min(16)]);

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

        // Send the transaction
        // zingolib's send method typically takes: address, amount, memo
        let txid = self.client
            .send(vec![(to_address, amount_zatoshis, memo)])
            .await
            .map_err(|e| {
                FaucetError::TransactionFailed(format!("Failed to send transaction: {}", e))
            })?;

        // Record in history
        self.history.add_transaction(TransactionRecord {
            txid: txid.clone(),
            recipient: to_address.to_string(),
            amount: amount_zec,
            timestamp: chrono::Utc::now(),
        })?;

        Ok(txid)
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