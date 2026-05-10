/// Metadata and connection info for a single chain.
/// Implement this trait for each supported chain in `domain/chains/`.
pub trait ChainInfo: Send + Sync {
    fn chain_id(&self) -> u64;
    fn name(&self) -> &'static str;
    fn rpc_url(&self) -> &'static str;
    fn explorer_url(&self) -> &'static str;

    fn tx_url(&self, tx_hash: &str) -> String {
        format!("{}/tx/{}", self.explorer_url(), tx_hash)
    }

    fn address_url(&self, address: &str) -> String {
        format!("{}/address/{}", self.explorer_url(), address)
    }
}
