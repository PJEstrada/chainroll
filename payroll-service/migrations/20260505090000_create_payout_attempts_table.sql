CREATE TABLE payout_attempts (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    payrun_id TEXT NOT NULL REFERENCES payruns(id) ON DELETE CASCADE,
    payout_instruction_id TEXT NOT NULL REFERENCES payout_instructions(id) ON DELETE CASCADE,
    attempt_number INTEGER NOT NULL,
    status TEXT NOT NULL,
    provider TEXT NOT NULL,
    signer_provider TEXT NOT NULL,
    provider_reference TEXT,
    transaction_hash TEXT,
    error_message TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    CONSTRAINT payout_attempts_attempt_number_chk CHECK (attempt_number > 0),
    CONSTRAINT payout_attempts_started_completion_chk CHECK (
        (
            status = 'started'
            AND completed_at IS NULL
            AND provider_reference IS NULL
            AND transaction_hash IS NULL
            AND error_message IS NULL
        )
        OR (
            status <> 'started'
            AND completed_at IS NOT NULL
        )
    ),
    CONSTRAINT payout_attempts_submitted_reference_chk CHECK (
        status <> 'submitted' OR provider_reference IS NOT NULL
    ),
    CONSTRAINT payout_attempts_failed_error_chk CHECK (
        status <> 'failed' OR error_message IS NOT NULL
    ),
    CONSTRAINT payout_attempts_review_error_chk CHECK (
        status <> 'review_required' OR error_message IS NOT NULL
    )
);

CREATE UNIQUE INDEX payout_attempts_instruction_attempt_number_idx
    ON payout_attempts (payout_instruction_id, attempt_number);

CREATE UNIQUE INDEX payout_attempts_one_started_instruction_idx
    ON payout_attempts (tenant_id, payout_instruction_id)
    WHERE status = 'started';

CREATE UNIQUE INDEX payout_attempts_one_submitted_instruction_idx
    ON payout_attempts (tenant_id, payout_instruction_id)
    WHERE status = 'submitted';

CREATE INDEX payout_attempts_payrun_id_idx
    ON payout_attempts (tenant_id, payrun_id);

CREATE INDEX payout_attempts_instruction_id_idx
    ON payout_attempts (tenant_id, payout_instruction_id);
