CREATE TABLE payout_instructions (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    payrun_id TEXT NOT NULL REFERENCES payruns(id) ON DELETE CASCADE,
    payrun_item_id TEXT NOT NULL REFERENCES payrun_items(id) ON DELETE CASCADE,
    employee_id TEXT NOT NULL,
    treasury_account_id TEXT NOT NULL,
    idempotency_key TEXT NOT NULL UNIQUE,
    destination_wallet_address TEXT NOT NULL,
    source_wallet_address TEXT NOT NULL,
    chain TEXT NOT NULL,
    chain_id BIGINT NOT NULL,
    token_symbol TEXT NOT NULL,
    token_address TEXT NOT NULL,
    token_decimals SMALLINT NOT NULL,
    amount_units NUMERIC(39, 0) NOT NULL,
    custody_provider TEXT NOT NULL,
    control_mode TEXT NOT NULL,
    provider_wallet_id TEXT,
    provider_owner_id TEXT,
    secret_reference TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT payout_instructions_amount_positive_chk CHECK (amount_units > 0),
    CONSTRAINT payout_instructions_token_decimals_chk CHECK (
        token_decimals >= 0 AND token_decimals <= 36
    )
);

CREATE UNIQUE INDEX payout_instructions_payrun_item_idx
    ON payout_instructions (payrun_item_id);
CREATE INDEX payout_instructions_payrun_id_idx
    ON payout_instructions (tenant_id, payrun_id);
CREATE INDEX payout_instructions_employee_id_idx
    ON payout_instructions (tenant_id, employee_id);
