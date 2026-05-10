use crate::Result;
use crate::error::Error;
use crate::services::datastore::PayoutAttemptStore;
use crate::services::datastore::postgres::payout_attempt_store::PayoutAttemptStoreError;
use crate::services::payout_instruction::service::PayoutInstructionService;
use crate::services::payout_submission::submit;
use crate::services::payout_submission::submit::{SubmitPayoutsRequest, SubmitPayoutsResponse};
use crate::services::stablecoin::client::StablecoinPayoutClient;
use error_stack::Report;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait PayoutSubmissionService {
    async fn submit(&self, req: SubmitPayoutsRequest) -> Result<SubmitPayoutsResponse>;
}

#[derive(Debug, Clone)]
pub struct PayoutSubmissionServiceImpl<PI, A, C>
where
    PI: PayoutInstructionService,
    A: PayoutAttemptStore,
    C: StablecoinPayoutClient,
{
    payout_instruction_service: PI,
    payout_attempt_store: A,
    stablecoin_client: C,
}

impl<PI, A, C> PayoutSubmissionServiceImpl<PI, A, C>
where
    PI: PayoutInstructionService,
    A: PayoutAttemptStore,
    C: StablecoinPayoutClient,
{
    pub fn new(
        payout_instruction_service: PI,
        payout_attempt_store: A,
        stablecoin_client: C,
    ) -> Self {
        Self {
            payout_instruction_service,
            payout_attempt_store,
            stablecoin_client,
        }
    }

    pub fn payout_instruction_service(&self) -> &PI {
        &self.payout_instruction_service
    }

    pub fn payout_attempt_store(&self) -> &A {
        &self.payout_attempt_store
    }

    pub fn stablecoin_client(&self) -> &C {
        &self.stablecoin_client
    }
}

impl<PI, A, C> PayoutSubmissionService for PayoutSubmissionServiceImpl<PI, A, C>
where
    PI: PayoutInstructionService,
    A: PayoutAttemptStore,
    C: StablecoinPayoutClient,
{
    async fn submit(&self, req: SubmitPayoutsRequest) -> Result<SubmitPayoutsResponse> {
        submit::execute(self, req).await
    }
}

pub(super) fn map_attempt_store_error(err: PayoutAttemptStoreError) -> Report<Error> {
    match err {
        PayoutAttemptStoreError::PayoutAttemptNotFound => Report::new(Error::NotFound),
        PayoutAttemptStoreError::InvalidId(_)
        | PayoutAttemptStoreError::InvalidStatus(_)
        | PayoutAttemptStoreError::InvalidProvider(_)
        | PayoutAttemptStoreError::InvalidSignerProvider(_)
        | PayoutAttemptStoreError::InvalidPayoutAttempt(_)
        | PayoutAttemptStoreError::InvalidAttemptNumber(_) => {
            Report::new(Error::InvalidInput(err.to_string()))
        }
        PayoutAttemptStoreError::Database(_) | PayoutAttemptStoreError::Audit(_) => {
            Report::new(err).change_context(Error::Database)
        }
    }
}
