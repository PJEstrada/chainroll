use alloy::primitives::{Address, Bytes, U256};
use alloy::sol;
use alloy::sol_types::SolCall;

sol! {
    function balanceOf(address owner) external view returns (uint256 balance);
    function transfer(address to, uint256 amount) external returns (bool);
}

/// ABI-encoded calldata for `balanceOf(owner)`.
pub fn balance_of_calldata(owner: Address) -> Bytes {
    balanceOfCall { owner }.abi_encode().into()
}

/// Decode the raw bytes returned by an `eth_call` to `balanceOf`.
/// ABI-encoded uint256 is always 32 bytes, big-endian.
pub fn decode_u256_return(raw: &[u8]) -> U256 {
    if raw.len() < 32 {
        return U256::ZERO;
    }
    U256::from_be_slice(&raw[..32])
}

/// ABI-encoded calldata for `transfer(to, amount)`.
pub fn transfer_calldata(to: Address, amount: U256) -> Bytes {
    transferCall { to, amount }.abi_encode().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::address;

    #[test]
    fn test_balance_of_calldata_selector() {
        let owner = address!("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266");
        let data = balance_of_calldata(owner);
        assert_eq!(
            data.len(),
            4 + 32,
            "should be selector (4) + padded address (32)"
        );
        assert_eq!(&data[..4], &balanceOfCall::SELECTOR, "selector must match");
    }

    #[test]
    fn test_transfer_calldata_selector() {
        let to = address!("70997970C51812dc3A010C7d01b50e0d17dc79C8");
        let amount = U256::from(1_000_000u64);
        let data = transfer_calldata(to, amount);
        assert_eq!(
            data.len(),
            4 + 32 + 32,
            "should be selector (4) + to (32) + amount (32)"
        );
        assert_eq!(&data[..4], &transferCall::SELECTOR, "selector must match");
    }

    #[test]
    fn test_decode_u256_return_full() {
        let mut raw = [0u8; 32];
        raw[31] = 42;
        assert_eq!(decode_u256_return(&raw), U256::from(42u64));
    }

    #[test]
    fn test_decode_u256_return_short_returns_zero() {
        assert_eq!(decode_u256_return(&[1, 2, 3]), U256::ZERO);
    }

    #[test]
    fn test_decode_u256_return_large_value() {
        let val = U256::from(1_000_000_000_000u64);
        let mut raw = [0u8; 32];
        raw[24..32].copy_from_slice(&1_000_000_000_000u64.to_be_bytes());
        assert_eq!(decode_u256_return(&raw), val);
    }
}
