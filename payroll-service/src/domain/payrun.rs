use crate::domain::compensation::CompensationAmount;
use crate::domain::employee::IDEmployee;
use crate::domain::ids::{IDResource, StandardID};
use crate::domain::tenant::IDTenant;
use crate::domain::treasury::TokenSymbol;
use chrono::{DateTime, Utc};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash, Serialize, Deserialize)]
pub struct IDPayrun;

impl IDResource for IDPayrun {
    fn prefix() -> Option<String> {
        Some("payrun".to_string())
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash, Serialize, Deserialize)]
pub struct IDPayrunItem;

impl IDResource for IDPayrunItem {
    fn prefix() -> Option<String> {
        Some("payrun_item".to_string())
    }
}

#[derive(Debug, Clone)]
pub struct Payrun {
    id: StandardID<IDPayrun>,
    tenant_id: StandardID<IDTenant>,
    status: PayrunStatus,
    items: Vec<PayrunItem>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Serialize for Payrun {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Payrun", 7)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("tenant_id", &self.tenant_id)?;
        state.serialize_field("status", &self.status)?;
        state.serialize_field("items", &self.items)?;
        state.serialize_field("totals", &self.totals().map_err(serde::ser::Error::custom)?)?;
        state.serialize_field("created_at", &self.created_at)?;
        state.serialize_field("updated_at", &self.updated_at)?;
        state.end()
    }
}

impl Payrun {
    pub fn new(preview: PayrunPreview, options: CreatePayrunOptions) -> Result<Self, PayrunError> {
        let now = Utc::now();
        let items = build_payrun_items(&preview, options)?;

        Self::restore(
            StandardID::new(),
            *preview.tenant_id(),
            PayrunStatus::Created,
            items,
            now,
            now,
        )
    }

    pub fn restore(
        id: StandardID<IDPayrun>,
        tenant_id: StandardID<IDTenant>,
        status: PayrunStatus,
        items: Vec<PayrunItem>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, PayrunError> {
        validate_payrun_items(&items)?;

        Ok(Self {
            id,
            tenant_id,
            status,
            items,
            created_at,
            updated_at,
        })
    }

    pub fn id(&self) -> &StandardID<IDPayrun> {
        &self.id
    }

    pub fn tenant_id(&self) -> &StandardID<IDTenant> {
        &self.tenant_id
    }

    pub fn status(&self) -> PayrunStatus {
        self.status
    }

    pub fn items(&self) -> &[PayrunItem] {
        &self.items
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn totals(&self) -> Result<PayrunPreviewTotals, PayrunPreviewError> {
        PayrunPreview::new(
            self.tenant_id,
            self.items
                .iter()
                .map(PayrunPreviewItem::from)
                .collect::<Vec<_>>(),
        )
        .totals()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PayrunItem {
    id: StandardID<IDPayrunItem>,
    employee_id: StandardID<IDEmployee>,
    status: PayrunItemStatus,
    amount: Option<CompensationAmount>,
    blockers: Vec<PayrunPreviewBlocker>,
}

impl PayrunItem {
    pub fn payable(employee_id: StandardID<IDEmployee>, amount: CompensationAmount) -> Self {
        Self {
            id: StandardID::new(),
            employee_id,
            status: PayrunItemStatus::Payable,
            amount: Some(amount),
            blockers: Vec::new(),
        }
    }

    pub fn excluded(
        employee_id: StandardID<IDEmployee>,
        amount: Option<CompensationAmount>,
        blockers: Vec<PayrunPreviewBlocker>,
    ) -> Result<Self, PayrunError> {
        if blockers.is_empty() {
            return Err(PayrunError::ExcludedItemRequiresBlocker);
        }

        Ok(Self {
            id: StandardID::new(),
            employee_id,
            status: PayrunItemStatus::Excluded,
            amount,
            blockers,
        })
    }

    pub fn restore(
        id: StandardID<IDPayrunItem>,
        employee_id: StandardID<IDEmployee>,
        status: PayrunItemStatus,
        amount: Option<CompensationAmount>,
        blockers: Vec<PayrunPreviewBlocker>,
    ) -> Result<Self, PayrunError> {
        validate_payrun_item(status, amount.as_ref(), &blockers)?;

        Ok(Self {
            id,
            employee_id,
            status,
            amount,
            blockers,
        })
    }

    pub fn id(&self) -> &StandardID<IDPayrunItem> {
        &self.id
    }

    pub fn employee_id(&self) -> &StandardID<IDEmployee> {
        &self.employee_id
    }

    pub fn status(&self) -> PayrunItemStatus {
        self.status
    }

    pub fn amount(&self) -> Option<&CompensationAmount> {
        self.amount.as_ref()
    }

    pub fn blockers(&self) -> &[PayrunPreviewBlocker] {
        &self.blockers
    }
}

impl From<&PayrunItem> for PayrunPreviewItem {
    fn from(item: &PayrunItem) -> Self {
        PayrunPreviewItem::new(item.employee_id, item.amount.clone(), item.blockers.clone())
            .expect("persisted payrun items must be valid preview items")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayrunStatus {
    Created,
}

impl Display for PayrunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PayrunStatus::Created => write!(f, "created"),
        }
    }
}

impl FromStr for PayrunStatus {
    type Err = ParsePayrunStatusError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "created" => Ok(Self::Created),
            other => Err(ParsePayrunStatusError(other.to_string())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid payrun status: {0}")]
pub struct ParsePayrunStatusError(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayrunItemStatus {
    Payable,
    Excluded,
}

impl Display for PayrunItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PayrunItemStatus::Payable => write!(f, "payable"),
            PayrunItemStatus::Excluded => write!(f, "excluded"),
        }
    }
}

impl FromStr for PayrunItemStatus {
    type Err = ParsePayrunItemStatusError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "payable" => Ok(Self::Payable),
            "excluded" => Ok(Self::Excluded),
            other => Err(ParsePayrunItemStatusError(other.to_string())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid payrun item status: {0}")]
pub struct ParsePayrunItemStatusError(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreatePayrunOptions {
    strict: bool,
    exclude_unpayable: bool,
}

impl CreatePayrunOptions {
    pub fn new(strict: bool, exclude_unpayable: bool) -> Self {
        Self {
            strict,
            exclude_unpayable,
        }
    }

    pub fn strict() -> Self {
        Self {
            strict: true,
            exclude_unpayable: false,
        }
    }

    pub fn exclude_unpayable() -> Self {
        Self {
            strict: false,
            exclude_unpayable: true,
        }
    }

    pub fn is_strict(self) -> bool {
        self.strict
    }

    pub fn excludes_unpayable(self) -> bool {
        self.exclude_unpayable
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PayrunError {
    #[error("strict payrun creation requires every preview item to be payable")]
    StrictModeRequiresReadyPreview,
    #[error("unpayable preview items require exclude_unpayable=true")]
    UnpayableItemsRequireExclusion,
    #[error("payrun must include at least one payable item")]
    NoPayableItems,
    #[error("payable payrun item requires an amount")]
    PayableItemRequiresAmount,
    #[error("payable payrun item cannot have blockers")]
    PayableItemCannotHaveBlockers,
    #[error("excluded payrun item requires at least one blocker")]
    ExcludedItemRequiresBlocker,
}

fn build_payrun_items(
    preview: &PayrunPreview,
    options: CreatePayrunOptions,
) -> Result<Vec<PayrunItem>, PayrunError> {
    if options.is_strict() && preview.status() != PayrunPreviewStatus::Ready {
        return Err(PayrunError::StrictModeRequiresReadyPreview);
    }

    if !options.excludes_unpayable()
        && preview.items().iter().any(|item| !item.is_payable())
        && preview.status() != PayrunPreviewStatus::Ready
    {
        return Err(PayrunError::UnpayableItemsRequireExclusion);
    }

    let mut items = Vec::with_capacity(preview.items().len());
    for item in preview.items() {
        if item.is_payable() {
            let amount = item
                .amount()
                .cloned()
                .ok_or(PayrunError::PayableItemRequiresAmount)?;
            items.push(PayrunItem::payable(*item.employee_id(), amount));
        } else if options.excludes_unpayable() {
            items.push(PayrunItem::excluded(
                *item.employee_id(),
                item.amount().cloned(),
                item.blockers().to_vec(),
            )?);
        }
    }

    Ok(items)
}

fn validate_payrun_items(items: &[PayrunItem]) -> Result<(), PayrunError> {
    if !items
        .iter()
        .any(|item| item.status == PayrunItemStatus::Payable)
    {
        return Err(PayrunError::NoPayableItems);
    }

    for item in items {
        validate_payrun_item(item.status, item.amount.as_ref(), &item.blockers)?;
    }

    Ok(())
}

fn validate_payrun_item(
    status: PayrunItemStatus,
    amount: Option<&CompensationAmount>,
    blockers: &[PayrunPreviewBlocker],
) -> Result<(), PayrunError> {
    match status {
        PayrunItemStatus::Payable => {
            if amount.is_none() {
                return Err(PayrunError::PayableItemRequiresAmount);
            }
            if !blockers.is_empty() {
                return Err(PayrunError::PayableItemCannotHaveBlockers);
            }
        }
        PayrunItemStatus::Excluded => {
            if blockers.is_empty() {
                return Err(PayrunError::ExcludedItemRequiresBlocker);
            }
        }
    }

    Ok(())
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

    #[test]
    fn strict_payrun_creation_freezes_ready_preview() {
        let tenant_id = StandardID::new();
        let employee_id = StandardID::new();
        let preview = PayrunPreview::new(
            tenant_id,
            vec![PayrunPreviewItem::payable(
                employee_id,
                amount(1_000_000, "USDC"),
            )],
        );

        let payrun = Payrun::new(preview, CreatePayrunOptions::strict()).unwrap();

        assert_eq!(payrun.tenant_id(), &tenant_id);
        assert_eq!(payrun.status(), PayrunStatus::Created);
        assert_eq!(payrun.items().len(), 1);
        assert_eq!(payrun.items()[0].employee_id(), &employee_id);
        assert_eq!(payrun.items()[0].status(), PayrunItemStatus::Payable);
        assert_eq!(
            payrun.totals().unwrap().total_amounts()[0].amount_units(),
            1_000_000
        );
    }

    #[test]
    fn strict_payrun_creation_rejects_blocked_preview() {
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

        let err = Payrun::new(preview, CreatePayrunOptions::strict()).unwrap_err();

        assert_eq!(err, PayrunError::StrictModeRequiresReadyPreview);
    }

    #[test]
    fn non_strict_payrun_creation_requires_exclusion_for_blocked_items() {
        let preview = PayrunPreview::new(
            StandardID::new(),
            vec![
                PayrunPreviewItem::payable(StandardID::new(), amount(1_000_000, "USDC")),
                PayrunPreviewItem::blocked(
                    StandardID::new(),
                    None,
                    vec![PayrunPreviewBlocker::MissingActiveCompensationProfile],
                )
                .unwrap(),
            ],
        );

        let err = Payrun::new(preview, CreatePayrunOptions::new(false, false)).unwrap_err();

        assert_eq!(err, PayrunError::UnpayableItemsRequireExclusion);
    }

    #[test]
    fn exclude_unpayable_freezes_payable_and_excluded_items() {
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

        let payrun = Payrun::new(preview, CreatePayrunOptions::exclude_unpayable()).unwrap();

        assert_eq!(payrun.items().len(), 2);
        assert_eq!(payrun.items()[0].status(), PayrunItemStatus::Payable);
        assert_eq!(payrun.items()[1].status(), PayrunItemStatus::Excluded);
        assert_eq!(
            payrun.items()[1].blockers(),
            &[PayrunPreviewBlocker::MissingWallet]
        );
        assert_eq!(
            payrun.totals().unwrap().total_amounts()[0].amount_units(),
            1_000_000
        );
    }

    #[test]
    fn payrun_creation_rejects_no_payable_items() {
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

        let err = Payrun::new(preview, CreatePayrunOptions::exclude_unpayable()).unwrap_err();

        assert_eq!(err, PayrunError::NoPayableItems);
    }
}
