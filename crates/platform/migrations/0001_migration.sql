CREATE TABLE IF NOT EXISTS acme_audit (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    domain TEXT NOT NULL,
    method TEXT NOT NULL,
    tenant_id TEXT NOT NULL,
    payload BYTEA NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS acme_audit_tenant ON acme_audit (tenant_id, id);
