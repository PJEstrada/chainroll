use mockall::automock;
use privy_rs::PrivyClient;
use privy_rs::generated::types::{CreateWalletBody, Wallet};

pub const APP_ID_ENV_VAR: &str = "PRIVY_TEST_APP_ID";
pub const APP_SECRET_ENV_VAR: &str = "PRIVY_TEST_APP_SECRET";

#[automock]
#[allow(async_fn_in_trait)]
pub trait WalletService {
    async fn create_wallet(&self, request: CreateWalletBody) -> anyhow::Result<Wallet>;
}
pub struct WalletsManager {
    client: PrivyClient,
}

impl WalletsManager {
    pub fn new(client: PrivyClient) -> Self {
        Self { client }
    }
}

impl WalletService for WalletsManager {
    async fn create_wallet(&self, request: CreateWalletBody) -> anyhow::Result<Wallet> {
        let wallet = self
            .client
            .wallets()
            .create(
                None, // idempotency key
                &request,
            )
            .await?
            .into_inner();
        Ok(wallet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use privy_rs::generated::types::{
        CreateWalletBody, Wallet, WalletAdditionalSigner, WalletChainType,
    };

    fn mock_wallet() -> Wallet {
        Wallet {
            id: "wallet-123".to_string(),
            address: "0xdeadbeef".to_string(),
            chain_type: WalletChainType::Ethereum,
            created_at: 0.0,
            exported_at: None,
            imported_at: None,
            owner_id: None,
            policy_ids: vec![],
            additional_signers: WalletAdditionalSigner(vec![]),
            public_key: None,
        }
    }

    #[tokio::test]
    async fn test_create_wallet() {
        let mut mock = MockWalletService::new();

        mock.expect_create_wallet()
            .once()
            .returning(|_| Ok(mock_wallet()));

        let request = CreateWalletBody {
            chain_type: WalletChainType::Ethereum,
            additional_signers: None,
            owner: None,
            owner_id: None,
            policy_ids: vec![],
        };

        let result = mock.create_wallet(request).await.unwrap();
        assert_eq!(result.id, "wallet-123");
        assert_eq!(result.address, "0xdeadbeef");
    }

    #[tokio::test]
    async fn test_create_wallet_error() {
        let mut mock = MockWalletService::new();

        mock.expect_create_wallet()
            .once()
            .returning(|_| Err(anyhow::anyhow!("api error")));

        let result = mock
            .create_wallet(CreateWalletBody {
                chain_type: WalletChainType::Ethereum,
                additional_signers: None,
                owner: None,
                owner_id: None,
                policy_ids: vec![],
            })
            .await;

        assert!(result.is_err());
    }
}
