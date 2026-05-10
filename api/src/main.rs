mod app_state;
mod application;
mod routes;
mod utils;

use crate::app_state::AppState;
use application::Application;
use payroll_service::services::compensation::service::CompensationServiceImpl;
use payroll_service::services::datastore::postgres::compensation_store::PgCompensationStore;
use payroll_service::services::datastore::postgres::employee_store::PgEmployeeStore;
use payroll_service::services::datastore::postgres::payout_attempt_store::PgPayoutAttemptStore;
use payroll_service::services::datastore::postgres::payout_instruction_store::PgPayoutInstructionStore;
use payroll_service::services::datastore::postgres::payrun_store::PgPayrunStore;
use payroll_service::services::datastore::postgres::treasury_store::PgTreasuryStore;
use payroll_service::services::employee::service::EmployeeServiceImpl;
use payroll_service::services::payout_instruction::service::PayoutInstructionServiceImpl;
use payroll_service::services::payout_submission::service::PayoutSubmissionServiceImpl;
use payroll_service::services::payrun::service::PayrunServiceImpl;
use payroll_service::services::stablecoin::tempo_privy::TempoPrivyStablecoinPayoutClient;
use payroll_service::services::treasury::service::TreasuryServiceImpl;
use secrecy::{ExposeSecret, SecretString};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use utils::settings::{DATABASE_URL, prod};

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();
    let pg_pool = configure_postgresql().await;
    let employee_store = PgEmployeeStore::new(pg_pool.clone());
    let payrun_employee_store = employee_store.clone();
    let payout_instruction_employee_store = employee_store.clone();
    let treasury_store = PgTreasuryStore::new(pg_pool.clone());
    let payrun_treasury_store = treasury_store.clone();
    let payout_instruction_treasury_store = treasury_store.clone();
    let compensation_store = PgCompensationStore::new(pg_pool.clone());
    let payrun_compensation_store = compensation_store.clone();
    let payrun_store = PgPayrunStore::new(pg_pool.clone());
    let payout_instruction_payrun_store = payrun_store.clone();
    let payout_instruction_store = PgPayoutInstructionStore::new(pg_pool.clone());
    let payout_attempt_store = PgPayoutAttemptStore::new(pg_pool);
    let employee_service = EmployeeServiceImpl::new(employee_store);
    let treasury_service = TreasuryServiceImpl::new(treasury_store);
    let compensation_service = CompensationServiceImpl::new(compensation_store);
    let payrun_service = PayrunServiceImpl::new(
        payrun_employee_store,
        payrun_compensation_store,
        payrun_treasury_store,
        payrun_store,
    );
    let payout_instruction_service = PayoutInstructionServiceImpl::new(
        payout_instruction_employee_store,
        payout_instruction_treasury_store,
        payout_instruction_payrun_store,
        payout_instruction_store,
    );
    let stablecoin_client = TempoPrivyStablecoinPayoutClient::from_env()
        .expect("Failed to configure Tempo Privy stablecoin payout client");
    let payout_submission_service = PayoutSubmissionServiceImpl::new(
        payout_instruction_service.clone(),
        payout_attempt_store,
        stablecoin_client,
    );
    let app_state = AppState::new(
        employee_service,
        treasury_service,
        compensation_service,
        payrun_service,
        payout_instruction_service,
        payout_submission_service,
    );

    let app_address = prod::app_address();
    let app = Application::build(app_state, &app_address)
        .await
        .expect("Failed to build application");

    tracing::info!("listening on {}", app.address);

    if let Err(e) = app.run().await {
        eprintln!("server error: {}", e);
    }
}

async fn configure_postgresql() -> PgPool {
    // Create a new database connection pool
    let pg_pool = get_postgres_pool(&DATABASE_URL)
        .await
        .expect("Failed to create Postgres connection pool!");

    // Run database migrations against our test database
    sqlx::migrate!("../payroll-service/migrations")
        .run(&pg_pool)
        .await
        .expect("Failed to run migrations");

    pg_pool
}

pub async fn get_postgres_pool(url: &SecretString) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(20)
        .connect(url.expose_secret())
        .await
}
