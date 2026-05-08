use crate::Result;
use crate::error::Error;
use crate::services::datastore::postgres::payout_instruction_store::PayoutInstructionStoreError;
use crate::services::datastore::postgres::payrun_store::PayrunStoreError;
use crate::services::datastore::{
    EmployeeStore, PayoutInstructionStore, PayrunStore, TreasuryStore,
};
use crate::services::payout_instruction::generate;
use crate::services::payout_instruction::generate::{GenerateRequest, GenerateResponse};
use error_stack::Report;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait PayoutInstructionService {
    async fn generate(&self, req: GenerateRequest) -> Result<GenerateResponse>;
}

#[derive(Debug, Clone)]
pub struct PayoutInstructionServiceImpl<E, T, P, I>
where
    E: EmployeeStore,
    T: TreasuryStore,
    P: PayrunStore,
    I: PayoutInstructionStore,
{
    employee_store: E,
    treasury_store: T,
    payrun_store: P,
    payout_instruction_store: I,
}

impl<E: EmployeeStore, T: TreasuryStore, P: PayrunStore, I: PayoutInstructionStore>
    PayoutInstructionServiceImpl<E, T, P, I>
{
    pub fn new(
        employee_store: E,
        treasury_store: T,
        payrun_store: P,
        payout_instruction_store: I,
    ) -> Self {
        Self {
            employee_store,
            treasury_store,
            payrun_store,
            payout_instruction_store,
        }
    }

    pub fn employee_store(&self) -> &E {
        &self.employee_store
    }

    pub fn treasury_store(&self) -> &T {
        &self.treasury_store
    }

    pub fn payrun_store(&self) -> &P {
        &self.payrun_store
    }

    pub fn payout_instruction_store(&self) -> &I {
        &self.payout_instruction_store
    }
}

impl<E: EmployeeStore, T: TreasuryStore, P: PayrunStore, I: PayoutInstructionStore>
    PayoutInstructionService for PayoutInstructionServiceImpl<E, T, P, I>
{
    async fn generate(&self, req: GenerateRequest) -> Result<GenerateResponse> {
        generate::execute(self, req).await
    }
}

pub(super) fn map_payrun_store_error(err: PayrunStoreError) -> Report<Error> {
    match err {
        PayrunStoreError::InvalidId(_)
        | PayrunStoreError::InvalidPayrunStatus(_)
        | PayrunStoreError::InvalidPayrunItemStatus(_)
        | PayrunStoreError::InvalidTokenSymbol(_)
        | PayrunStoreError::InvalidAmountUnits
        | PayrunStoreError::InvalidPayrun(_) => Report::new(Error::InvalidInput(err.to_string())),
        PayrunStoreError::Database(_) | PayrunStoreError::Audit(_) => {
            Report::new(err).change_context(Error::Database)
        }
    }
}

pub(super) fn map_instruction_store_error(err: PayoutInstructionStoreError) -> Report<Error> {
    match err {
        PayoutInstructionStoreError::InvalidId(_)
        | PayoutInstructionStoreError::InvalidWalletAddress(_)
        | PayoutInstructionStoreError::InvalidTokenSymbol(_)
        | PayoutInstructionStoreError::InvalidChain(_)
        | PayoutInstructionStoreError::InvalidCustodyProvider(_)
        | PayoutInstructionStoreError::InvalidControlMode(_)
        | PayoutInstructionStoreError::InvalidPayoutInstruction(_)
        | PayoutInstructionStoreError::InvalidCompensationAmount(_)
        | PayoutInstructionStoreError::InvalidAmountUnits
        | PayoutInstructionStoreError::InvalidChainId(_)
        | PayoutInstructionStoreError::InvalidTokenDecimals(_)
        | PayoutInstructionStoreError::AuditEventCountMismatch => {
            Report::new(Error::InvalidInput(err.to_string()))
        }
        PayoutInstructionStoreError::Database(_) | PayoutInstructionStoreError::Audit(_) => {
            Report::new(err).change_context(Error::Database)
        }
    }
}
