-- CREATE EXTENSION IF NOT EXISTS citext;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
-- ======================
-- HELPER FUNCTION
-- ======================
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
WHEN duplicate_object THEN NULL;
END $$;
DO $$ BEGIN CREATE TYPE user_role AS ENUM ('admin', 'regular');
EXCEPTION
WHEN duplicate_object THEN NULL;
END $$;
DO $$ BEGIN CREATE TYPE user_status AS ENUM (
    'active',
    'suspended',
    'pending_verification'
);
EXCEPTION
WHEN duplicate_object THEN NULL;
END $$;
DO $$ BEGIN CREATE TYPE transaction_type AS ENUM (
    'initial_credit',
    'usage_charge',
    'top_up',
    'refund'
);
EXCEPTION
WHEN duplicate_object THEN NULL;
END $$;
DO $$ BEGIN CREATE TYPE provider AS ENUM ('google', 'github', 'email');
EXCEPTION
WHEN duplicate_object THEN NULL;
END $$;
-- ======================
-- USERS
-- ======================
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    full_name VARCHAR(200) NOT NULL,
    username VARCHAR(64) NOT NULL,
    email VARCHAR(255) NOT NULL,
    password TEXT,
    -- nullable for OAuth-only users
    phone_number VARCHAR(32),
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
-- ======================
-- USER WALLETS (1:1)
-- ======================
CREATE TABLE IF NOT EXISTS user_wallets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    credit_balance NUMERIC(18, 6) NOT NULL DEFAULT 0.000000,
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TRIGGER set_user_wallets_timestamp BEFORE
UPDATE ON user_wallets FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();
CREATE OR REPLACE FUNCTION give_free_credit() RETURNS TRIGGER AS $$
DECLARE new_wallet_id UUID;
initial_credit_amount NUMERIC(18, 6);
BEGIN -- 1. Create the wallet with a ZERO balance
INSERT INTO user_wallets (user_id, credit_balance)
VALUES (NEW.id, 0.00)
RETURNING id INTO new_wallet_id;
-- 2. Calculate the random credit
initial_credit_amount := ((floor(random() * 91) + 10)::numeric)::NUMERIC(18, 6);
-- 3. Insert the "initial_credit" transaction into the ledger
INSERT INTO wallet_transactions (wallet_id, amount, "type", details)
VALUES (
        new_wallet_id,
        initial_credit_amount,
        'initial_credit',
        'Initial sign-up bonus'
    );
-- 4. The `apply_wallet_transaction` trigger will
--    now automatically and safely update the wallet's
--    balance from 0 to the initial_credit_amount.
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
    name VARCHAR(150) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (owner_id, name)
);
CREATE TRIGGER set_projects_timestamp BEFORE
UPDATE ON projects FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();
-- ======================
-- DEPLOYMENTS
-- ======================
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
-- ======================
-- DEPLOYMENT EVENTS
-- ======================
CREATE TABLE IF NOT EXISTS deployment_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    deployment_id UUID NOT NULL REFERENCES deployments(id) ON DELETE CASCADE,
    event_type VARCHAR(128) NOT NULL,
    message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_deployment_events_deployment_id ON deployment_events(deployment_id);
-- ======================
-- BILLING RECORDS (Usage logs)
-- ======================
CREATE TABLE IF NOT EXISTS billing_records (
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
        charged_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_billing_records_user_id ON billing_records(user_id);
CREATE INDEX IF NOT EXISTS idx_billing_records_deployment_id ON billing_records(deployment_id);
-- ======================
-- WALLET TRANSACTIONS (Ledger)
-- ======================
CREATE TABLE IF NOT EXISTS wallet_transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wallet_id UUID NOT NULL REFERENCES user_wallets(id) ON DELETE CASCADE,
    amount NUMERIC(18, 6) NOT NULL,
    "type" transaction_type NOT NULL,
    details TEXT,
    billing_record_id UUID REFERENCES billing_records(id) ON DELETE
    SET NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_wallet_transactions_wallet_id ON wallet_transactions(wallet_id);
CREATE OR REPLACE FUNCTION apply_wallet_transaction() RETURNS TRIGGER AS $$
DECLARE current_balance NUMERIC(18, 6);
new_balance NUMERIC(18, 6);
BEGIN
SELECT credit_balance INTO current_balance
FROM user_wallets
WHERE id = NEW.wallet_id FOR
UPDATE;
IF NOT FOUND THEN RAISE EXCEPTION 'Wallet % not found',
NEW.wallet_id;
END IF;
new_balance := (current_balance + NEW.amount);
IF new_balance < 0 THEN RAISE EXCEPTION 'Insufficient funds: wallet % would go negative (current=%). Transaction aborted.',
NEW.wallet_id,
current_balance;
END IF;
UPDATE user_wallets
SET credit_balance = new_balance,
    updated_at = NOW()
WHERE id = NEW.wallet_id;
RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER wallet_transaction_after_insert
AFTER
INSERT ON wallet_transactions FOR EACH ROW EXECUTE PROCEDURE apply_wallet_transaction();