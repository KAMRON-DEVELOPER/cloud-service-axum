-- ==============================================
-- EXTENSIONS
-- ==============================================
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS citext;
--
--
-- ==============================================
-- HELPER FUNCTION
-- ==============================================
CREATE OR REPLACE FUNCTION trigger_set_timestamp() RETURNS TRIGGER AS $$ BEGIN NEW.updated_at = NOW();
RETURN NEW;
END;
$$ LANGUAGE plpgsql;
--
--
-- ==============================================
-- ENUM TYPES
-- ==============================================
DO $$ BEGIN CREATE TYPE deployment_status AS ENUM (
    'pending',
    'running',
    'succeeded',
    'failed',
    'terminated'
);
EXCEPTION
WHEN duplicate_object THEN NULL;
END $$;
DO $$ BEGIN CREATE TYPE user_role AS ENUM ('admin', 'regular');
EXCEPTION
WHEN duplicate_object THEN NULL;
END $$;
DO $$ BEGIN CREATE TYPE user_status AS ENUM ('active', 'suspended', 'pending_verification');
EXCEPTION
WHEN duplicate_object THEN NULL;
END $$;
DO $$ BEGIN CREATE TYPE transaction_type AS ENUM ('free_credit', 'usage_charge', 'fund');
EXCEPTION
WHEN duplicate_object THEN NULL;
END $$;
DO $$ BEGIN CREATE TYPE provider AS ENUM ('google', 'github', 'email');
EXCEPTION
WHEN duplicate_object THEN NULL;
END $$;
--
--
-- ==============================================
-- USERS
-- ==============================================
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    full_name VARCHAR(200) NOT NULL,
    username VARCHAR(64) NOT NULL,
    email VARCHAR(255) NOT NULL,
    password TEXT,
    picture TEXT,
    email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    role user_role NOT NULL DEFAULT 'regular',
    status user_status NOT NULL DEFAULT 'pending_verification',
    oauth_user_id TEXT,
    last_login_at TIMESTAMPTZ,
    last_login_ip VARCHAR(45),
    deleted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (username)
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_users_lower_email ON users (lower(email));
CREATE TRIGGER set_users_timestamp BEFORE
UPDATE ON users FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();
--
--
-- ==============================================
-- BALANCES
-- ==============================================
CREATE TABLE IF NOT EXISTS balances (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    amount NUMERIC(18, 6) NOT NULL DEFAULT 0.000000,
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TRIGGER set_balances_timestamp BEFORE
UPDATE ON balances FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();
--
--
-- ==============================================
-- TRANSACTIONS
-- ==============================================
CREATE TABLE IF NOT EXISTS transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    balance_id UUID NOT NULL REFERENCES balances(id) ON DELETE CASCADE,
    amount NUMERIC(18, 6) NOT NULL,
    type transaction_type NOT NULL,
    detail TEXT,
    billing_id UUID REFERENCES billing_records(id) ON DELETE
    SET NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_transactions_balance_id ON transactions(balance_id);
--
--
-- ==============================================
-- APPLY TRANSACTION TO BALANCE
-- ==============================================
CREATE OR REPLACE FUNCTION apply_transaction() RETURNS TRIGGER AS $$
DECLARE current_balance NUMERIC(18, 6);
new_balance NUMERIC(18, 6);
BEGIN
SELECT amount INTO current_balance
FROM balances
WHERE id = NEW.balance_id FOR
UPDATE;
IF NOT FOUND THEN RAISE EXCEPTION 'Balance % not found',
NEW.balance_id;
END IF;
new_balance := (current_balance + NEW.amount);
IF new_balance < 0 THEN RAISE EXCEPTION 'Insufficient funds: balance % would go negative (current=%). Transaction aborted.',
NEW.balance_id,
current_balance;
END IF;
UPDATE balances
SET amount = new_balance,
    updated_at = NOW()
WHERE id = NEW.balance_id;
RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER after_transaction_insert
AFTER
INSERT ON transactions FOR EACH ROW EXECUTE PROCEDURE apply_transaction();
--
--
-- ==============================================
-- SYSTEM CONFIG (controls free credit dynamically)
-- ==============================================
CREATE TABLE IF NOT EXISTS system_config (
    id BOOLEAN PRIMARY KEY DEFAULT TRUE,
    free_credit_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    free_credit_amount NUMERIC(18, 6) NOT NULL DEFAULT 0.00,
    free_credit_detail TEXT DEFAULT 'Free credit',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
--
--
-- ==============================================
-- AUTO-BALANCE CREATION + OPTIONAL FREE CREDIT
-- ==============================================
CREATE OR REPLACE FUNCTION on_user_created_balance() RETURNS TRIGGER AS $$
DECLARE cfg RECORD;
balance_id UUID;
BEGIN -- Create empty balance for user
INSERT INTO balances (user_id, amount)
VALUES (NEW.id, 0.00)
RETURNING id INTO balance_id;
-- Load current system config
SELECT * INTO cfg
FROM system_config
LIMIT 1;
-- If free credit is enabled, apply it
IF cfg.free_credit_enabled
AND cfg.free_credit_amount > 0 THEN
INSERT INTO transactions (balance_id, amount, type, detail)
VALUES (
        balance_id,
        cfg.free_credit_amount,
        'free_credit',
        cfg.free_credit_detail
    );
END IF;
RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER after_user_created
AFTER
INSERT ON users FOR EACH ROW EXECUTE PROCEDURE on_user_created_balance();
--
--
-- ==============================================
-- PROJECTS
-- ==============================================
CREATE TABLE IF NOT EXISTS projects (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(150) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (owner_id, name)
);
CREATE TRIGGER set_projects_timestamp BEFORE
UPDATE ON projects FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();
--
--
-- ==============================================
-- DEPLOYMENTS
-- ==============================================
CREATE TABLE IF NOT EXISTS deployments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name VARCHAR(128) NOT NULL,
    image TEXT NOT NULL,
    env_vars JSONB NOT NULL DEFAULT '{}'::jsonb,
    replicas INTEGER NOT NULL DEFAULT 1 CHECK (replicas >= 1),
    resources JSONB NOT NULL DEFAULT jsonb_build_object(
        'cpu_request_millicores',
        250,
        'cpu_limit_millicores',
        500,
        'memory_request_mb',
        256,
        'memory_limit_mb',
        512
    ),
    labels JSONB,
    status deployment_status NOT NULL DEFAULT 'pending',
    cluster_namespace VARCHAR(128) NOT NULL DEFAULT 'default',
    cluster_deployment_name VARCHAR(192) NOT NULL,
    node_selector JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (project_id, name)
);
CREATE INDEX IF NOT EXISTS idx_deployments_user_id ON deployments(user_id);
CREATE INDEX IF NOT EXISTS idx_deployments_project_id ON deployments(project_id);
CREATE INDEX IF NOT EXISTS idx_deployments_status ON deployments(status);
CREATE TRIGGER set_deployments_timestamp BEFORE
UPDATE ON deployments FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();
--
--
-- ==============================================
-- DEPLOYMENT SECRETS (application-managed encryption)
-- ==============================================
CREATE TABLE IF NOT EXISTS deployment_secrets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    deployment_id UUID NOT NULL REFERENCES deployments(id) ON DELETE CASCADE,
    key TEXT NOT NULL,
    value BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_deployment_secret_key ON deployment_secrets (deployment_id, key);
--
--
-- ==============================================
-- DEPLOYMENT EVENTS
-- ==============================================
CREATE TABLE IF NOT EXISTS deployment_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    deployment_id UUID NOT NULL REFERENCES deployments(id) ON DELETE CASCADE,
    event_type VARCHAR(128) NOT NULL,
    message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_deployment_events_deployment_id ON deployment_events(deployment_id);
--
--
-- ==============================================
-- BILLINGS
-- ==============================================
CREATE TABLE IF NOT EXISTS billings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    deployment_id UUID REFERENCES deployments(id) ON DELETE
    SET NULL,
        resources_snapshot JSONB NOT NULL,
        cpu_millicores INTEGER NOT NULL CHECK (cpu_millicores >= 0),
        memory_mb INTEGER NOT NULL CHECK (memory_mb >= 0),
        cost_per_hour NUMERIC(18, 8) NOT NULL,
        hours_used NUMERIC(12, 6) NOT NULL DEFAULT 1.0,
        total_cost NUMERIC(20, 8) GENERATED ALWAYS AS (cost_per_hour * hours_used) STORED,
        created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_billing_records_user_id ON billing_records(user_id);
CREATE INDEX IF NOT EXISTS idx_billing_records_deployment_id ON billing_records(deployment_id);