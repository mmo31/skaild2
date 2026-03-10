CREATE TABLE IF NOT EXISTS routes (
    id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    application_id  UUID         NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    host            VARCHAR(255) NOT NULL,
    path_prefix     VARCHAR(255) NOT NULL DEFAULT '/',
    access_mode     VARCHAR(50)  NOT NULL DEFAULT 'login_required',
    enabled         BOOLEAN      NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    CONSTRAINT routes_access_mode_check CHECK (access_mode IN ('public', 'login_required'))
);

-- Enforce unique host+path per application
CREATE UNIQUE INDEX idx_routes_host_path ON routes(application_id, host, path_prefix);

-- Index for gateway route lookup by host
CREATE INDEX idx_routes_host ON routes(host);
