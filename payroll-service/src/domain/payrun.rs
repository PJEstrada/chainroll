use crate::domain::compensation::CompensationAmount;
use crate::domain::employee::IDEmployee;
use crate::domain::ids::StandardID;
use crate::domain::tenant::IDTenant;
use crate::domain::treasury::TokenSymbol;
use serde::Serialize;
use serde::ser::SerializeStruct;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct PayrunPreview {
    tenant_id: StandardID<IDTenant>,
    items: Vec<PayrunPreviewItem>,
}

impl Serialize for PayrunPreview {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("PayrunPreview", 4)?;
        state.serialize_field("tenant_id", &self.tenant_id)?;
        state.serialize_field("status", &self.status())?;
        state.serialize_field("items", &self.items)?;
        state.serialize_field("totals", &self.totals().map_err(serde::ser::Error::custom)?)?;
        state.end()
    }
}

impl PayrunPreview {
    pub fn new(tenant_id: StandardID<IDTenant>, items: Vec<PayrunPreviewItem>) -> Self {
        Self { tenant_id, items }
    }

    pub fn tenant_id(&self) -> &StandardID<IDTenant> {
        &self.tenant_id
    }

    pub fn items(&self) -> &[PayrunPreviewItem] {
        &self.items
    }

    pub fn status(&self) -> PayrunPreviewStatus {
        let payable = self.items.iter().filter(|item| item.is_payable()).count();

        if payable == 0 {
            return PayrunPreviewStatus::Blocked;
        }

        if payable == self.items.len() {
            return PayrunPreviewStatus::Ready;
        }

        PayrunPreviewStatus::PartiallyReady
    }

    pub fn totals(&self) -> Result<PayrunPreviewTotals, PayrunPreviewError> {
        Ok(PayrunPreviewTotals {
            total_amounts: self.total_amounts()?,
            total_blockers: self.items.iter().map(|item| item.blockers().len()).sum(),
            total_employees: self.items.len(),
            total_employees_with_blockers: self
                .items
                .iter()
                .filter(|item| item.has_blockers())
                .count(),
            total_employees_without_blockers: self
                .items
                .iter()
                .filter(|item| !item.has_blockers())
                .count(),
        })
    }

    fn total_amounts(&self) -> Result<Vec<CompensationAmount>, PayrunPreviewError> {
        let mut totals = BTreeMap::<String, (TokenSymbol, u128)>::new();

        for amount in self
            .items
            .iter()
            .filter_map(PayrunPreviewItem::payable_amount)
        {
            let key = amount.token_symbol().as_str().to_string();
            let entry = totals
                .entry(key)
                .or_insert_with(|| (amount.token_symbol().clone(), 0));
            entry.1 = entry
                .1
                .checked_add(amount.amount_units())
                .ok_or(PayrunPreviewError::TotalAmountOverflow)?;
        }

        totals
            .into_values()
            .map(|(token_symbol, amount_units)| {
                CompensationAmount::new(amount_units, token_symbol)
                    .map_err(|_| PayrunPreviewError::InvalidTotalAmount)
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PayrunPreviewItem {
    employee_id: StandardID<IDEmployee>,
    amount: Option<CompensationAmount>,
    blockers: Vec<PayrunPreviewBlocker>,
}

impl PayrunPreviewItem {
    pub fn new(
        employee_id: StandardID<IDEmployee>,
        amount: Option<CompensationAmount>,
        blockers: Vec<PayrunPreviewBlocker>,
    ) -> Result<Self, PayrunPreviewError> {
        if amount.is_none() && blockers.is_empty() {
            return Err(PayrunPreviewError::ItemRequiresAmountOrBlocker);
        }

        Ok(Self {
            employee_id,
            amount,
            blockers,
        })
    }

    pub fn payable(employee_id: StandardID<IDEmployee>, amount: CompensationAmount) -> Self {
        Self {
            employee_id,
            amount: Some(amount),
            blockers: Vec::new(),
        }
    }

    pub fn blocked(
        employee_id: StandardID<IDEmployee>,
        amount: Option<CompensationAmount>,
        blockers: Vec<PayrunPreviewBlocker>,
    ) -> Result<Self, PayrunPreviewError> {
        if blockers.is_empty() {
            return Err(PayrunPreviewError::BlockedItemRequiresBlocker);
        }

        Ok(Self {
            employee_id,
            amount,
            blockers,
        })
    }

    pub fn employee_id(&self) -> &StandardID<IDEmployee> {
        &self.employee_id
    }

    pub fn amount(&self) -> Option<&CompensationAmount> {
        self.amount.as_ref()
    }

    pub fn blockers(&self) -> &[PayrunPreviewBlocker] {
        &self.blockers
    }

    pub fn has_blockers(&self) -> bool {
        !self.blockers.is_empty()
    }

    pub fn is_payable(&self) -> bool {
        self.amount.is_some() && self.blockers.is_empty()
    }

    fn payable_amount(&self) -> Option<&CompensationAmount> {
        if self.is_payable() {
            return self.amount.as_ref();
        }

        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PayrunPreviewBlocker {
    MissingWallet,
    MissingActiveCompensationProfile,
    MissingTreasuryAccount,
    TokenMismatch,
    TreasuryRequiresUserSignature,
}

#[derive(Debug, Clone, Serialize)]
pub struct PayrunPreviewTotals {
    total_amounts: Vec<CompensationAmount>,
    total_blockers: usize,
    total_employees: usize,
    total_employees_with_blockers: usize,
    total_employees_without_blockers: usize,
}

impl PayrunPreviewTotals {
    pub fn total_amounts(&self) -> &[CompensationAmount] {
        &self.total_amounts
    }

    pub fn total_blockers(&self) -> usize {
        self.total_blockers
    }

    pub fn total_employees(&self) -> usize {
        self.total_employees
    }

    pub fn total_employees_with_blockers(&self) -> usize {
        self.total_employees_with_blockers
    }

    pub fn total_employees_without_blockers(&self) -> usize {
        self.total_employees_without_blockers
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PayrunPreviewStatus {
    Blocked,
    PartiallyReady,
    Ready,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PayrunPreviewError {
    #[error("payrun preview item requires either an amount or a blocker")]
    ItemRequiresAmountOrBlocker,
    #[error("blocked payrun preview item requires at least one blocker")]
    BlockedItemRequiresBlocker,
    #[error("payrun preview total amount overflow")]
    TotalAmountOverflow,
    #[error("payrun preview total amount is invalid")]
    InvalidTotalAmount,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::treasury::TokenSymbol;

    fn amount(amount_units: u128, token_symbol: &str) -> CompensationAmount {
        CompensationAmount::new(amount_units, TokenSymbol::parse(token_symbol).unwrap()).unwrap()
    }

    #[test]
    fn payable_item_has_amount_and_no_blockers() {
        let employee_id = StandardID::new();
        let item = PayrunPreviewItem::payable(employee_id, amount(1_000_000, "USDC"));

        assert_eq!(item.employee_id(), &employee_id);
        assert!(item.is_payable());
        assert!(!item.has_blockers());
        assert_eq!(item.amount().unwrap().amount_units(), 1_000_000);
    }

    #[test]
    fn blocked_item_requires_a_blocker() {
        let err = PayrunPreviewItem::blocked(StandardID::new(), None, Vec::new()).unwrap_err();

        assert_eq!(err, PayrunPreviewError::BlockedItemRequiresBlocker);
    }

    #[test]
    fn generic_item_requires_amount_or_blocker() {
        let err = PayrunPreviewItem::new(StandardID::new(), None, Vec::new()).unwrap_err();

        assert_eq!(err, PayrunPreviewError::ItemRequiresAmountOrBlocker);
    }

    #[test]
    fn preview_is_ready_when_every_item_is_payable() {
        let preview = PayrunPreview::new(
            StandardID::new(),
            vec![
                PayrunPreviewItem::payable(StandardID::new(), amount(1_000_000, "USDC")),
                PayrunPreviewItem::payable(StandardID::new(), amount(2_000_000, "USDC")),
            ],
        );

        assert_eq!(preview.status(), PayrunPreviewStatus::Ready);
    }

    #[test]
    fn preview_is_partially_ready_when_some_items_are_blocked() {
        let preview = PayrunPreview::new(
            StandardID::new(),
            vec![
                PayrunPreviewItem::payable(StandardID::new(), amount(1_000_000, "USDC")),
                PayrunPreviewItem::blocked(
                    StandardID::new(),
                    Some(amount(2_000_000, "USDC")),
                    vec![PayrunPreviewBlocker::MissingWallet],
                )
                .unwrap(),
            ],
        );

        assert_eq!(preview.status(), PayrunPreviewStatus::PartiallyReady);
    }

    #[test]
    fn preview_is_blocked_when_no_items_are_payable() {
        let preview = PayrunPreview::new(
            StandardID::new(),
            vec![
                PayrunPreviewItem::blocked(
                    StandardID::new(),
                    None,
                    vec![PayrunPreviewBlocker::MissingActiveCompensationProfile],
                )
                .unwrap(),
            ],
        );

        assert_eq!(preview.status(), PayrunPreviewStatus::Blocked);
    }

    #[test]
    fn preview_is_blocked_when_it_has_no_items() {
        let preview = PayrunPreview::new(StandardID::new(), Vec::new());

        assert_eq!(preview.status(), PayrunPreviewStatus::Blocked);
    }

    #[test]
    fn totals_count_employees_and_blockers() {
        let preview = PayrunPreview::new(
            StandardID::new(),
            vec![
                PayrunPreviewItem::payable(StandardID::new(), amount(1_000_000, "USDC")),
                PayrunPreviewItem::blocked(
                    StandardID::new(),
                    Some(amount(2_000_000, "USDC")),
                    vec![
                        PayrunPreviewBlocker::MissingWallet,
                        PayrunPreviewBlocker::TreasuryRequiresUserSignature,
                    ],
                )
                .unwrap(),
                PayrunPreviewItem::blocked(
                    StandardID::new(),
                    None,
                    vec![PayrunPreviewBlocker::MissingActiveCompensationProfile],
                )
                .unwrap(),
            ],
        );

        let totals = preview.totals().unwrap();

        assert_eq!(totals.total_blockers(), 3);
        assert_eq!(totals.total_employees(), 3);
        assert_eq!(totals.total_employees_with_blockers(), 2);
        assert_eq!(totals.total_employees_without_blockers(), 1);
    }

    #[test]
    fn totals_group_payable_amounts_by_token() {
        let preview = PayrunPreview::new(
            StandardID::new(),
            vec![
                PayrunPreviewItem::payable(StandardID::new(), amount(1_000_000, "USDC")),
                PayrunPreviewItem::payable(StandardID::new(), amount(2_000_000, "USDC")),
                PayrunPreviewItem::payable(StandardID::new(), amount(3_000_000, "pathUSD")),
            ],
        );

        let totals = preview.totals().unwrap();

        assert_eq!(totals.total_amounts().len(), 2);
        assert_eq!(totals.total_amounts()[0].token_symbol().as_str(), "USDC");
        assert_eq!(totals.total_amounts()[0].amount_units(), 3_000_000);
        assert_eq!(totals.total_amounts()[1].token_symbol().as_str(), "pathUSD");
        assert_eq!(totals.total_amounts()[1].amount_units(), 3_000_000);
    }

    #[test]
    fn totals_exclude_blocked_amounts() {
        let preview = PayrunPreview::new(
            StandardID::new(),
            vec![
                PayrunPreviewItem::payable(StandardID::new(), amount(1_000_000, "USDC")),
                PayrunPreviewItem::blocked(
                    StandardID::new(),
                    Some(amount(9_000_000, "USDC")),
                    vec![PayrunPreviewBlocker::TokenMismatch],
                )
                .unwrap(),
            ],
        );

        let totals = preview.totals().unwrap();

        assert_eq!(totals.total_amounts().len(), 1);
        assert_eq!(totals.total_amounts()[0].amount_units(), 1_000_000);
    }

    #[test]
    fn serializes_with_derived_status_and_totals() {
        let preview = PayrunPreview::new(
            StandardID::new(),
            vec![PayrunPreviewItem::payable(
                StandardID::new(),
                amount(1_000_000, "USDC"),
            )],
        );

        let body = serde_json::to_value(preview).unwrap();

        assert_eq!(body["status"], "ready");
        assert_eq!(body["totals"]["total_employees"], 1);
        assert_eq!(
            body["totals"]["total_amounts"][0]["amount_units"],
            "1000000"
        );
    }
}
