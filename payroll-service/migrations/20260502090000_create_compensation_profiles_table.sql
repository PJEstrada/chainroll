CREATE TABLE compensation_profiles (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    employee_id TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('active', 'inactive')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    amount_units NUMERIC(39, 0) NOT NULL CHECK (amount_units > 0),
    token_symbol TEXT NOT NULL,
    compensation_cadence TEXT NOT NULL CHECK (
        compensation_cadence IN ('weekly', 'biweekly', 'monthly', 'custom')
    ),
    compensation_cadence_every INTEGER,
    compensation_cadence_unit TEXT,
    valid_from TIMESTAMPTZ,
    valid_to TIMESTAMPTZ,
    CONSTRAINT compensation_profile_token_symbol_not_empty CHECK (length(trim(token_symbol)) > 0),
    CONSTRAINT compensation_profile_custom_cadence_config CHECK (
        (
            compensation_cadence = 'custom'
            AND compensation_cadence_every IS NOT NULL
            AND compensation_cadence_every > 0
            AND compensation_cadence_unit IN ('day', 'week', 'month')
        )
        OR (
            compensation_cadence != 'custom'
            AND compensation_cadence_every IS NULL
            AND compensation_cadence_unit IS NULL
        )
    ),
    CONSTRAINT compensation_profile_validity_window CHECK (
        valid_to IS NULL
        OR valid_from IS NULL
        OR valid_to > valid_from
    )
);

CREATE INDEX compensation_profiles_tenant_employee_idx
    ON compensation_profiles (tenant_id, employee_id);

CREATE UNIQUE INDEX compensation_profiles_one_active_per_employee_idx
    ON compensation_profiles (tenant_id, employee_id)
    WHERE status = 'active';
