CREATE TABLE IF NOT EXISTS employees (
    id          TEXT PRIMARY KEY,
    tenant_id   TEXT NOT NULL,
    identifier  TEXT NOT NULL,
    first_name  TEXT NOT NULL,
    last_name   TEXT NOT NULL,
    divisions   JSONB NOT NULL DEFAULT '[]',
    culture     TEXT,
    attributes  JSONB,
    status      TEXT NOT NULL DEFAULT 'Active',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_employees_tenant_id ON employees (tenant_id);