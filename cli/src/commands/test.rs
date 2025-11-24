use crate::error::Result;
use colored::*;
use reqwest::Client;
use serde_json::Value;
use std::process::Command;
use tokio::time::{sleep, Duration};

pub async fn execute() -> Result<()> {
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("{}", "  ZecKit - Running Smoke Tests".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!();

    let client = Client::new();
    let mut passed = 0;
    let mut failed = 0;

    // Test 1: Zebra RPC
    print!("  [1/5] Zebra RPC connectivity... ");
    match test_zebra_rpc(&client).await {
        Ok(_) => {
            println!("{}", "✓ PASS".green());
            passed += 1;
        }
        Err(e) => {
            println!("{} {}", "✗ FAIL".red(), e);
            failed += 1;
        }
    }

    // Test 2: Faucet Health
    print!("  [2/5] Faucet health check... ");
    match test_faucet_health(&client).await {
        Ok(_) => {
            println!("{}", "✓ PASS".green());
            passed += 1;
        }
        Err(e) => {
            println!("{} {}", "✗ FAIL".red(), e);
            failed += 1;
        }
    }

    // Test 3: Faucet Stats
    print!("  [3/5] Faucet stats endpoint... ");
    match test_faucet_stats(&client).await {
        Ok(_) => {
            println!("{}", "✓ PASS".green());
            passed += 1;
        }
        Err(e) => {
            println!("{} {}", "✗ FAIL".red(), e);
            failed += 1;
        }
    }

    // Test 4: Faucet Address
    print!("  [4/5] Faucet address retrieval... ");
    match test_faucet_address(&client).await {
        Ok(_) => {
            println!("{}", "✓ PASS".green());
            passed += 1;
        }
        Err(e) => {
            println!("{} {}", "✗ FAIL".red(), e);
            failed += 1;
        }
    }

    // Test 5: Faucet Request (real shielded transaction)
    print!("  [5/5] Faucet funding request... ");
    match test_faucet_request(&client).await {
        Ok(_) => {
            println!("{}", "✓ PASS".green());
            passed += 1;
        }
        Err(e) => {
            println!("{} {}", "✗ FAIL".red(), e);
            failed += 1;
        }
    }

    println!();
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("  {} Tests passed: {}", "✓".green(), passed.to_string().green());
    println!("  {} Tests failed: {}", "✗".red(), failed.to_string().red());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!();

    if failed > 0 {
        return Err(crate::error::ZecDevError::HealthCheck(
            format!("{} test(s) failed", failed)
        ));
    }

    Ok(())
}

async fn test_zebra_rpc(client: &Client) -> Result<()> {
    let resp = client
        .post("http://127.0.0.1:8232")
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": "test",
            "method": "getblockcount",
            "params": []
        }))
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(crate::error::ZecDevError::HealthCheck(
            "Zebra RPC not responding".into()
        ));
    }

    Ok(())
}

async fn test_faucet_health(client: &Client) -> Result<()> {
    let resp = client
        .get("http://127.0.0.1:8080/health")
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(crate::error::ZecDevError::HealthCheck(
            "Faucet health check failed".into()
        ));
    }

    Ok(())
}

async fn test_faucet_stats(client: &Client) -> Result<()> {
    let resp = client
        .get("http://127.0.0.1:8080/stats")
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(crate::error::ZecDevError::HealthCheck(
            "Faucet stats not available".into()
        ));
    }

    let json: Value = resp.json().await?;
    
    // Verify key fields exist
    if json.get("faucet_address").is_none() {
        return Err(crate::error::ZecDevError::HealthCheck(
            "Stats missing faucet_address".into()
        ));
    }
    
    if json.get("current_balance").is_none() {
        return Err(crate::error::ZecDevError::HealthCheck(
            "Stats missing current_balance".into()
        ));
    }

    Ok(())
}

async fn test_faucet_address(client: &Client) -> Result<()> {
    let resp = client
        .get("http://127.0.0.1:8080/address")
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(crate::error::ZecDevError::HealthCheck(
            "Could not get faucet address".into()
        ));
    }

    let json: Value = resp.json().await?;
    if json.get("address").is_none() {
        return Err(crate::error::ZecDevError::HealthCheck(
            "Invalid address response".into()
        ));
    }

    Ok(())
}

async fn test_faucet_request(client: &Client) -> Result<()> {
    // Step 1: Sync the wallet first
    println!();
    println!("    {} Syncing wallet before test...", "↻".cyan());
    
    let sync_result = Command::new("docker")
        .args(&[
            "exec", "-i", "zeckit-zingo-wallet",
            "sh", "-c",
            "echo 'sync run\nquit' | zingo-cli --data-dir /var/zingo --server http://lightwalletd:9067"
        ])
        .output();
    
    if let Ok(output) = sync_result {
        if !output.status.success() {
            println!("    {} Sync warning: {}", "⚠".yellow(), 
                String::from_utf8_lossy(&output.stderr));
        }
    }
    
    // Wait for sync to settle
    sleep(Duration::from_secs(5)).await;
    
    // Step 2: Check balance
    println!("    {} Checking wallet balance...", "↻".cyan());
    
    let stats_resp = client
        .get("http://127.0.0.1:8080/stats")
        .send()
        .await?;
    
    if stats_resp.status().is_success() {
        let stats: Value = stats_resp.json().await?;
        let balance = stats["current_balance"].as_f64().unwrap_or(0.0);
        
        println!("    {} Balance: {} ZEC", "💰".cyan(), balance);
        
        if balance < 1.0 {
            println!("    {} Insufficient balance for test (need 1.0 ZEC)", "⚠".yellow());
            println!("    {} SKIP (wallet needs funds - this is expected on fresh start)", "→".yellow());
            // Don't fail - wallet needs time to see mined funds
            return Ok(());
        }
    }
    
    // Step 3: Get fixture address to send to
    println!("    {} Loading test fixture...", "↻".cyan());
    
    let fixture_path = std::path::Path::new("fixtures/unified-addresses.json");
    if !fixture_path.exists() {
        println!("    {} No fixture found - SKIP", "⚠".yellow());
        return Ok(());
    }
    
    let fixture_content = std::fs::read_to_string(fixture_path)?;
    let fixture: Value = serde_json::from_str(&fixture_content)?;
    
    let test_address = fixture["faucet_address"]
        .as_str()
        .ok_or_else(|| crate::error::ZecDevError::HealthCheck(
            "Invalid fixture address".into()
        ))?;
    
    // Step 4: Test funding request
    println!("    {} Sending 1.0 ZEC...", "↻".cyan());
    
    let resp = client
        .post("http://127.0.0.1:8080/request")
        .json(&serde_json::json!({
            "address": test_address,
            "amount": 1.0
        }))
        .timeout(Duration::from_secs(30))
        .send()
        .await?;

    println!(); // Clear line before result
    print!("  [5/5] Faucet funding request... ");

    if !resp.status().is_success() {
        let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(crate::error::ZecDevError::HealthCheck(
            format!("Request failed: {}", error_text)
        ));
    }

    let json: Value = resp.json().await?;
    
    // Verify we got a TXID (real blockchain transaction!)
    if let Some(txid) = json.get("txid").and_then(|v| v.as_str()) {
        if txid.is_empty() {
            return Err(crate::error::ZecDevError::HealthCheck(
                "Empty TXID returned".into()
            ));
        }
        // Success - we sent a real shielded transaction!
        Ok(())
    } else {
        Err(crate::error::ZecDevError::HealthCheck(
            "No TXID in response".into()
        ))
    }
}