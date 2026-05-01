CREATE TABLE IF NOT EXISTS audit_events (
    id          TEXT PRIMARY KEY,
    tenant_id   TEXT NOT NULL,
    actor_id    TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id   TEXT NOT NULL,
    event_type  TEXT NOT NULL,
    payload     JSONB NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_audit_events_tenant_created_at
    ON audit_events (tenant_id, created_at);

CREATE INDEX IF NOT EXISTS idx_audit_events_tenant_entity
    ON audit_events (tenant_id, entity_type, entity_id);
