-- Enable UUID support
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
-- ======================
-- HELPER FUNCTION
-- ======================
-- Function to update 'updated_at' timestamp
CREATE OR REPLACE FUNCTION trigger_set_timestamp() RETURNS TRIGGER AS $$ BEGIN NEW.updated_at = NOW();
RETURN NEW;
END;
$$ LANGUAGE plpgsql;
-- ======================
-- ENUM TYPES
-- ======================
DO $$ BEGIN CREATE TYPE deployment_status AS ENUM (
    'pending',
    'running',
    'succeeded',
    'failed',
    'terminated'
);
EXCEPTION
WHEN duplicate_object THEN null;
END $$;
DO $$ BEGIN CREATE TYPE user_role AS ENUM ('admin', 'regular');
EXCEPTION
WHEN duplicate_object THEN null;
END $$;
DO $$ BEGIN CREATE TYPE user_status AS ENUM (
    'active',
    'suspended',
    'pending_verification'
);
EXCEPTION
WHEN duplicate_object THEN null;
END $$;
DO $$ BEGIN CREATE TYPE transaction_type AS ENUM (
    'initial_credit',
    'usage_charge',
    'top_up',
    'refund'
);
EXCEPTION
WHEN duplicate_object THEN null;
END $$;
-- ======================
-- USERS
-- (Synced with services/users/src/features/models.rs)
-- ======================
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    full_name VARCHAR(100) NOT NULL,
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password TEXT,
    -- Nullable for OAuth-only users
    picture TEXT,
    email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    -- Fields from your Rust model
    role user_role NOT NULL DEFAULT 'regular',
    status user_status NOT NULL DEFAULT 'pending_verification',
    oauth_user_id TEXT,
    -- e.g., 'google_sub' or 'github_id'
    last_login_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    last_login_ip VARCHAR(45),
    -- Supports IPv6
    deleted_at TIMESTAMPTZ,
    -- For soft deletes
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
CREATE TRIGGER set_users_timestamp BEFORE
UPDATE ON users FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();
-- ======================
-- USER WALLETS (Replaces billing_accounts)
-- (Simplified 1-to-1 mapping with users for credit)
-- ======================
CREATE TABLE IF NOT EXISTS user_wallets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    -- Using NUMERIC(12, 4) for high-precision currency
    credit_balance NUMERIC(12, 4) NOT NULL DEFAULT 0.00,
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
CREATE TRIGGER set_user_wallets_timestamp BEFORE
UPDATE ON user_wallets FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();
-- Give new users $10-$100 free credit
CREATE OR REPLACE FUNCTION give_free_credit() RETURNS TRIGGER AS $$ BEGIN
INSERT INTO user_wallets (user_id, credit_balance)
VALUES (
        NEW.id,
        -- (10.00 to 100.99) -> floor(random() * 91) + 10
        (floor(random() * 91) + 10)::NUMERIC(12, 4)
    );
RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER on_user_created
AFTER
INSERT ON users FOR EACH ROW EXECUTE PROCEDURE give_free_credit();
-- ======================
-- PROJECTS
-- ======================
CREATE TABLE IF NOT EXISTS projects (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    -- Made nullable to match Option<String>
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (owner_id, name)
);
CREATE TRIGGER set_projects_timestamp BEFORE
UPDATE ON projects FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();
-- ======================
-- DEPLOYMENTS
-- (Synced with services/compute/src/features/models.rs)
-- ======================
CREATE TABLE IF NOT EXISTS deployments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name VARCHAR(64) NOT NULL,
    -- Name of the service (e.g., 'web', 'worker')
    image TEXT NOT NULL,
    env_vars JSONB DEFAULT '{}'::jsonb,
    replicas INTEGER NOT NULL DEFAULT 1,
    cpu_limit_millicores INTEGER NOT NULL DEFAULT 500,
    memory_limit_mb INTEGER NOT NULL DEFAULT 512,
    -- Correctly uses the ENUM type, not VARCHAR
    status deployment_status NOT NULL DEFAULT 'pending',
    cluster_namespace VARCHAR(64) NOT NULL DEFAULT 'default',
    cluster_deployment_name VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_deployments_user_id ON deployments(user_id);
CREATE INDEX IF NOT EXISTS idx_deployments_project_id ON deployments(project_id);
CREATE INDEX IF NOT EXISTS idx_deployments_status ON deployments(status);
CREATE TRIGGER set_deployments_timestamp BEFORE
UPDATE ON deployments FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();
-- ======================
-- BILLING RECORDS (Usage logs)
-- ======================
CREATE TABLE IF NOT EXISTS billing_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- Nullable, as some charges might not be for a specific deployment
    deployment_id UUID REFERENCES deployments(id) ON DELETE
    SET NULL,
        cpu_millicores INTEGER NOT NULL,
        memory_mb INTEGER NOT NULL,
        cost_per_hour NUMERIC(10, 6) NOT NULL,
        -- 6 decimal places for precision
        hours_used NUMERIC(10, 4) NOT NULL DEFAULT 1.0,
        total_cost NUMERIC(12, 6) GENERATED ALWAYS AS (cost_per_hour * hours_used) STORED,
        charged_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_billing_records_user_id ON billing_records(user_id);
CREATE INDEX IF NOT EXISTS idx_billing_records_deployment_id ON billing_records(deployment_id);
-- ======================
-- WALLET TRANSACTIONS (Financial ledger)
-- (Replaces transactions)
-- ======================
CREATE TABLE IF NOT EXISTS wallet_transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wallet_id UUID NOT NULL REFERENCES user_wallets(id) ON DELETE CASCADE,
    -- Can be positive (top-up) or negative (charge)
    amount NUMERIC(10, 4) NOT NULL,
    type transaction_type NOT NULL,
    details TEXT,
    -- Nullable, for extra info
    -- Link to the specific usage record that caused this charge
    billing_record_id UUID REFERENCES billing_records(id) ON DELETE
    SET NULL,
        created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_wallet_transactions_wallet_id ON wallet_transactions(wallet_id);
-- ======================
-- DEPLOYMENT EVENTS (Audit log)
-- ======================
CREATE TABLE IF NOT EXISTS deployment_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    deployment_id UUID NOT NULL REFERENCES deployments(id) ON DELETE CASCADE,
    event_type VARCHAR(64) NOT NULL,
    message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_deployment_events_deployment_id ON deployment_events(deployment_id);