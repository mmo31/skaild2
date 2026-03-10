-- Applications registered for proxying by skaild2
CREATE TABLE IF NOT EXISTS applications (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name         VARCHAR(255) NOT NULL,
    upstream_url TEXT         NOT NULL,
    hostname     VARCHAR(255) NOT NULL,
    enabled      BOOLEAN      NOT NULL DEFAULT TRUE,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- Hostname must be unique across all applications
CREATE UNIQUE INDEX idx_applications_hostname ON applications(hostname);
