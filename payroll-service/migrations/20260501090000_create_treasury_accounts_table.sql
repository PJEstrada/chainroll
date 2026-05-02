CREATE TABLE treasury_accounts (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    name TEXT NOT NULL,
    chain TEXT NOT NULL,
    chain_id BIGINT NOT NULL,
    token_symbol TEXT NOT NULL,
    token_address TEXT NOT NULL,
    token_decimals SMALLINT NOT NULL CHECK (token_decimals >= 0 AND token_decimals <= 36),
    sender_address TEXT NOT NULL,
    custody_provider TEXT NOT NULL,
    control_mode TEXT NOT NULL,
    provider_wallet_id TEXT,
    provider_owner_id TEXT,
    secret_reference TEXT,
    status TEXT NOT NULL CHECK (status IN ('active', 'inactive')),
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT treasury_account_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT treasury_account_token_symbol_not_empty CHECK (length(trim(token_symbol)) > 0),
    CONSTRAINT treasury_account_local_key_config CHECK (
        custody_provider != 'local_key'
        OR (
            control_mode = 'server_controlled'
            AND secret_reference IS NOT NULL
            AND length(trim(secret_reference)) > 0
        )
    ),
    CONSTRAINT treasury_account_privy_config CHECK (
        custody_provider != 'privy'
        OR (
            control_mode IN ('server_controlled', 'user_signature_required', 'user_delegated')
            AND provider_wallet_id IS NOT NULL
            AND length(trim(provider_wallet_id)) > 0
            AND (
                control_mode != 'user_delegated'
                OR (
                    provider_owner_id IS NOT NULL
                    AND length(trim(provider_owner_id)) > 0
                )
            )
        )
    ),
    CONSTRAINT treasury_account_external_config CHECK (
        custody_provider != 'external'
        OR control_mode = 'external_execution'
    ),
    CONSTRAINT treasury_default_requires_active CHECK (
        is_default = FALSE OR status = 'active'
    )
);

CREATE INDEX treasury_accounts_tenant_idx
    ON treasury_accounts (tenant_id);

CREATE UNIQUE INDEX treasury_accounts_one_default_active_per_token_idx
    ON treasury_accounts (tenant_id, chain, token_address)
    WHERE is_default = TRUE AND status = 'active';
