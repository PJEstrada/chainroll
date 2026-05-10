use crate::domain::EVM_ADDRESS_LEN;
use crate::domain::chain::Chain;

/// AccountRef is the stable handle the system uses to refer to an account
/// that lives in some custody/control system.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccountRef {
    pub id: AccountId,
    pub ctrl: AccountControl,
}

// represents the unique identifier of an account in a particular custody system.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AccountId {
    Local { name: String },
    Privy { wallet_id: String },
}

// AccountControl represents how the account is controlled (custody + signing model).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AccountControl {
    /// Local private key (useful for dev/demos).
    LocalKey,
    /// Privy: account controlled via Tempo custody providers
    TempoPrivy,
    /// Privy: account controlled via Tempo custody providers
    ArcPrivy,
}

/// Capability flags used by routing and execution decisions, this may grow over time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AccountCapabilities {
    /// the account can submit transactions using a sponsor/paymaster-like mechanism.
    pub can_sponsor_fees: bool,
    /// the custody system must submit the transaction (you cannot broadcast raw tx).
    pub requires_provider_submission: bool,
    /// the account supports batching (e.g., multi-call or native batching).
    pub supports_batching: bool,
}

/// Resolved account metadata returned by a custody/key-management system.
/// TODO: still need to figure out privy specifics, this might change drastically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountView {
    pub chain: Chain,
    pub address: [u8; EVM_ADDRESS_LEN],
    pub account_ref: AccountRef,
    pub control: AccountControl,
    pub caps: AccountCapabilities,
}
