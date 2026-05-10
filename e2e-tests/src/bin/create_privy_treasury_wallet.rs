use anyhow::{Context, Result};
use privy_rs::PrivyClient;
use privy_rs::generated::types::{CreateWalletBody, WalletChainType};

#[tokio::main]
async fn main() -> Result<()> {
    let app_id = env("PRIVY_APP_ID")?;
    let app_secret = env("PRIVY_APP_SECRET")?;
    let client = PrivyClient::new(app_id, app_secret).context("failed to create Privy client")?;
    let wallet = client
        .wallets()
        .create(
            None,
            &CreateWalletBody {
                chain_type: WalletChainType::Ethereum,
                additional_signers: None,
                owner: None,
                owner_id: None,
                policy_ids: vec![],
            },
        )
        .await
        .context("failed to create Privy wallet")?
        .into_inner();

    println!("Created Privy treasury wallet.");
    println!();
    println!("Add these values to e2e-tests/tempo-privy-payroll.env:");
    println!("TEMPO_PRIVY_WALLET_ID={}", wallet.id);
    println!("CHAINROLL_E2E_TREASURY_ADDRESS={}", wallet.address);
    println!();
    println!("Fund this address on Tempo testnet before running the full payroll e2e.");

    Ok(())
}

fn env(name: &str) -> Result<String> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .with_context(|| format!("missing env var: {name}"))
}
