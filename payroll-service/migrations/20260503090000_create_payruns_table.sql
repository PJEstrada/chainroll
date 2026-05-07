CREATE TABLE payruns (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX payruns_tenant_id_idx ON payruns (tenant_id);

CREATE TABLE payrun_items (
    id TEXT PRIMARY KEY,
    payrun_id TEXT NOT NULL REFERENCES payruns(id) ON DELETE CASCADE,
    tenant_id TEXT NOT NULL,
    employee_id TEXT NOT NULL,
    status TEXT NOT NULL,
    amount_units NUMERIC(39, 0),
    token_symbol TEXT,
    blockers JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT payrun_items_amount_pair_chk CHECK (
        (amount_units IS NULL AND token_symbol IS NULL)
        OR (amount_units IS NOT NULL AND token_symbol IS NOT NULL)
    ),
    CONSTRAINT payrun_items_status_chk CHECK (
        (status = 'payable' AND amount_units IS NOT NULL AND jsonb_array_length(blockers) = 0)
        OR (status = 'excluded' AND jsonb_array_length(blockers) > 0)
    )
);

CREATE INDEX payrun_items_payrun_id_idx ON payrun_items (payrun_id);
CREATE INDEX payrun_items_tenant_employee_idx ON payrun_items (tenant_id, employee_id);
