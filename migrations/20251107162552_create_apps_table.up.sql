CREATE TYPE deployment_status AS ENUM (
    'PENDING',    -- En attente de publication
    'PUBLISHING', -- Publication IPNS en cours
    'DEPLOYED',   -- En ligne
    'FAILED'      -- Échec
);

CREATE TABLE apps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    name VARCHAR(255) UNIQUE NOT NULL,
    key_name VARCHAR(255) UNIQUE NULL,
    ipns_name VARCHAR(255) UNIQUE NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER set_app_updated_at
BEFORE UPDATE ON apps
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_updated_at();

CREATE TABLE deployments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    app_id UUID NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
    cid VARCHAR(255) NOT NULL,
    status deployment_status NOT NULL DEFAULT 'PENDING',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_deployments_app_id ON deployments(app_id);

CREATE TYPE pin_status AS ENUM (
    'PINNING',    -- Le node a reçu l'ordre, mais n'a pas confirmé
    'PINNED',     -- Le node a confirmé avoir le contenu
    'FAILED'      -- Le node a échoué à pinner
);

CREATE TABLE deployments_nodes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    deployment_id UUID NOT NULL REFERENCES deployments(id) ON DELETE CASCADE,
    node_id UUID NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    status pin_status NOT NULL DEFAULT 'PINNING',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(deployment_id, node_id)
);

CREATE TRIGGER set_deployments_nodes_updated_at
BEFORE UPDATE ON deployments_nodes
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_updated_at();
