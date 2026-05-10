use anyhow::{Context, Result, anyhow, bail};
use reqwest::{Client, StatusCode};
use serde_json::{Value, json};
use std::time::Duration;

const PATH_USD_ADDRESS: &str = "0x20c0000000000000000000000000000000000000";

#[tokio::test]
#[ignore = "requires a running Chainroll API, funded Tempo Privy treasury wallet, and real recipient wallets"]
async fn tempo_privy_full_payroll_flow() -> Result<()> {
    let config = E2eConfig::from_env()?;
    let client = Client::builder()
        .timeout(Duration::from_secs(180))
        .build()
        .context("failed to build reqwest client")?;

    let treasury = post_json(
        &client,
        &config,
        "/treasury-accounts",
        json!({
            "name": format!("Tempo Privy E2E {}", config.tenant_id),
            "chain": "tempo-testnet",
            "token_symbol": &config.token_symbol,
            "token_address": &config.token_address,
            "token_decimals": config.token_decimals,
            "sender_address": &config.treasury_address,
            "custody_provider": "privy",
            "control_mode": "server_controlled",
            "provider_wallet_id": &config.privy_wallet_id,
            "is_default": true
        }),
        StatusCode::CREATED,
    )
    .await?;
    assert_eq!(treasury["custody_provider"], "privy");
    assert_eq!(treasury["control_mode"], "server_controlled");
    assert_eq!(treasury["is_default"], true);

    for (index, wallet) in config.employee_wallets.iter().enumerate() {
        let employee = post_json(
            &client,
            &config,
            "/employees",
            json!({
                "identifier": format!("E2E-{}-{}", config.tenant_id, index + 1),
                "first_name": format!("E2E{}", index + 1),
                "last_name": "Tempo",
                "wallet_address": wallet
            }),
            StatusCode::CREATED,
        )
        .await?;
        let employee_id = employee["id"]
            .as_str()
            .ok_or_else(|| anyhow!("employee response did not include id"))?
            .to_string();

        post_json(
            &client,
            &config,
            &format!("/employees/{employee_id}/compensation-profiles"),
            json!({
                "amount_units": &config.amount_units,
                "token_symbol": &config.token_symbol,
                "cadence": "monthly"
            }),
            StatusCode::CREATED,
        )
        .await?;
    }

    let preview = post_json(
        &client,
        &config,
        "/payruns/preview",
        json!({}),
        StatusCode::OK,
    )
    .await?;
    assert_eq!(preview["status"], "ready");
    assert_eq!(preview["totals"]["total_employees"], 3);
    assert_eq!(preview["totals"]["total_blockers"], 0);

    let payrun = post_json(
        &client,
        &config,
        "/payruns",
        json!({ "strict": true }),
        StatusCode::CREATED,
    )
    .await?;
    let payrun_id = payrun["id"]
        .as_str()
        .ok_or_else(|| anyhow!("payrun response did not include id"))?;
    assert_eq!(payrun["status"], "created");
    assert_eq!(payrun["items"].as_array().map(Vec::len), Some(3));

    let submission = post_json(
        &client,
        &config,
        &format!("/payruns/{payrun_id}/submit-payouts"),
        json!({}),
        StatusCode::OK,
    )
    .await?;
    assert_eq!(submission["total_instructions"], 3);
    assert_eq!(submission["failed"], 0);
    assert_eq!(submission["skipped"], 0);

    let submitted = submission["submitted"].as_u64().unwrap_or_default();
    let review_required = submission["review_required"].as_u64().unwrap_or_default();
    assert_eq!(submitted + review_required, 3);

    Ok(())
}

async fn post_json(
    client: &Client,
    config: &E2eConfig,
    path: &str,
    body: Value,
    expected_status: StatusCode,
) -> Result<Value> {
    let url = format!("{}{}", config.base_url, path);
    let response = client
        .post(url)
        .header("x-tenant-id", &config.tenant_id)
        .header("x-actor-id", &config.actor_id)
        .json(&body)
        .send()
        .await
        .with_context(|| format!("request failed for {path}"))?;
    let status = response.status();
    let text = response
        .text()
        .await
        .with_context(|| format!("failed to read response body for {path}"))?;

    if status != expected_status {
        bail!("expected {expected_status} from {path}, got {status}: {text}");
    }

    serde_json::from_str(&text)
        .with_context(|| format!("invalid JSON response from {path}: {text}"))
}

#[derive(Debug)]
struct E2eConfig {
    base_url: String,
    tenant_id: String,
    actor_id: String,
    privy_wallet_id: String,
    treasury_address: String,
    employee_wallets: Vec<String>,
    token_symbol: String,
    token_address: String,
    token_decimals: u8,
    amount_units: String,
}

impl E2eConfig {
    fn from_env() -> Result<Self> {
        require_real_execution_ack()?;

        let employee_wallets = env("CHAINROLL_E2E_EMPLOYEE_WALLETS")?
            .split(',')
            .map(str::trim)
            .filter(|wallet| !wallet.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();
        if employee_wallets.len() != 3 {
            bail!("CHAINROLL_E2E_EMPLOYEE_WALLETS must contain exactly 3 comma-separated wallets");
        }

        Ok(Self {
            base_url: env_or("CHAINROLL_E2E_BASE_URL", "http://127.0.0.1:8000")?
                .trim_end_matches('/')
                .to_string(),
            tenant_id: env_or_generated_tsid("CHAINROLL_E2E_TENANT_ID")?,
            actor_id: env_or_generated_tsid("CHAINROLL_E2E_ACTOR_ID")?,
            privy_wallet_id: env("TEMPO_PRIVY_WALLET_ID")?,
            treasury_address: env("CHAINROLL_E2E_TREASURY_ADDRESS")?,
            employee_wallets,
            token_symbol: env_or("CHAINROLL_E2E_TOKEN_SYMBOL", "pathUSD")?,
            token_address: env_or("CHAINROLL_E2E_TOKEN_ADDRESS", PATH_USD_ADDRESS)?,
            token_decimals: env_or("CHAINROLL_E2E_TOKEN_DECIMALS", "18")?
                .parse()
                .context("CHAINROLL_E2E_TOKEN_DECIMALS must be a u8")?,
            amount_units: env_or("CHAINROLL_E2E_AMOUNT_UNITS", "1")?,
        })
    }
}

fn require_real_execution_ack() -> Result<()> {
    let value = env_or("CHAINROLL_E2E_EXECUTE_REAL_PAYOUTS", "false")?;
    if value != "true" {
        bail!("set CHAINROLL_E2E_EXECUTE_REAL_PAYOUTS=true to run the real payout e2e test");
    }

    Ok(())
}

fn env(name: &str) -> Result<String> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .with_context(|| format!("missing env var: {name}"))
}

fn env_or(name: &str, default: &str) -> Result<String> {
    Ok(std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| default.to_string()))
}

fn env_or_generated_tsid(name: &str) -> Result<String> {
    Ok(std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| tsid::create_tsid().to_string()))
}
