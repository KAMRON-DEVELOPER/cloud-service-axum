-- Enable UUID support
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
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
DO $$ BEGIN CREATE TYPE plan_type AS ENUM ('free', 'basic', 'pro');
EXCEPTION
WHEN duplicate_object THEN null;
END $$;
-- ======================
-- USERS
-- ======================
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    full_name VARCHAR(100) NOT NULL,
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password TEXT,
    picture TEXT,
    email_verified BOOLEAN DEFAULT FALSE,
    credit_balance NUMERIC(12, 2) NOT NULL DEFAULT 50.00,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
-- ======================
-- BILLING ACCOUNTS
-- ======================
CREATE TABLE IF NOT EXISTS billing_accounts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plan plan_type NOT NULL DEFAULT 'free',
    credits NUMERIC(10, 2) NOT NULL DEFAULT 0,
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
-- Give new users some free credits
CREATE OR REPLACE FUNCTION give_free_credits() RETURNS TRIGGER AS $$ BEGIN
INSERT INTO billing_accounts (user_id, credits, plan)
VALUES (
        NEW.id,
        (10 + floor(random() * 90))::NUMERIC,
        'free'
    );
RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER on_user_created
AFTER
INSERT ON users FOR EACH ROW EXECUTE PROCEDURE give_free_credits();
-- ======================
-- BILLING TRANSACTIONS
-- ======================
CREATE TABLE IF NOT EXISTS billing_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    deployment_id UUID REFERENCES deployments(id) ON DELETE
    SET NULL,
        cpu_millicores INTEGER NOT NULL,
        memory_mb INTEGER NOT NULL,
        cost_per_hour NUMERIC(10, 4) NOT NULL,
        hours_used NUMERIC(10, 2) NOT NULL DEFAULT 1.0,
        total_cost NUMERIC(12, 4) GENERATED ALWAYS AS (cost_per_hour * hours_used) STORED,
        charged_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
-- ======================
-- BILLING TRANSACTIONS
-- ======================
CREATE TABLE IF NOT EXISTS transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    account_id UUID NOT NULL REFERENCES billing_accounts(id) ON DELETE CASCADE,
    amount NUMERIC(10, 2) NOT NULL,
    reason TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
-- ======================
-- COMPUTE PROJECTS
-- ======================
CREATE TABLE IF NOT EXISTS projects (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (owner_id, name)
);
-- ======================
-- DEPLOYMENTS (per-project)
-- ======================
CREATE TABLE IF NOT EXISTS deployments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(64) NOT NULL,
    image TEXT NOT NULL,
    env_vars JSONB DEFAULT '{}'::jsonb,
    replicas INTEGER NOT NULL DEFAULT 1,
    cpu_limit_millicores INTEGER NOT NULL DEFAULT 500,
    -- 0.5 CPU
    memory_limit_mb INTEGER NOT NULL DEFAULT 512,
    status VARCHAR(32) NOT NULL DEFAULT 'pending',
    -- pending | running | stopped | failed
    cluster_namespace VARCHAR(64) NOT NULL DEFAULT 'default',
    cluster_deployment_name VARCHAR(128) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_deployments_user_id ON deployments(user_id);
CREATE INDEX IF NOT EXISTS idx_deployments_status ON deployments(status);
CREATE TRIGGER set_deployments_timestamp BEFORE
UPDATE ON deployments FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();
-- ======================
-- JOBS (batch tasks / cron / on-demand compute)
-- ======================
CREATE TABLE IF NOT EXISTS deployment_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    deployment_id UUID NOT NULL REFERENCES deployments(id) ON DELETE CASCADE,
    event_type VARCHAR(64) NOT NULL,
    -- created, scaled, stopped, deleted, error, etc.
    message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_deployment_events_deployment_id ON deployment_events(deployment_id);
-- ======================
-- JOBS (batch tasks / cron / on-demand compute)
-- ======================
CREATE TABLE IF NOT EXISTS jobs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    image TEXT NOT NULL,
    command TEXT,
    status deployment_status NOT NULL DEFAULT 'pending',
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);