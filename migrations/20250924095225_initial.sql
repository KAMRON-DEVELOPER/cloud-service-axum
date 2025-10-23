-- Enable UUID support
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
-- =====================
-- ENUM TYPES (Idempotent Creation)
-- =====================
DO $$ BEGIN CREATE TYPE user_role AS ENUM ('admin', 'regular');
EXCEPTION
WHEN duplicate_object THEN null;
END $$;
DO $$ BEGIN CREATE TYPE user_status AS ENUM ('active', 'inactive');
EXCEPTION
WHEN duplicate_object THEN null;
END $$;
DO $$ BEGIN CREATE TYPE apartment_condition AS ENUM ('new', 'repaired', 'old');
EXCEPTION
WHEN duplicate_object THEN null;
END $$;
DO $$ BEGIN CREATE TYPE sale_type AS ENUM ('buy', 'rent');
EXCEPTION
WHEN duplicate_object THEN null;
END $$;
DO $$ BEGIN CREATE TYPE provider AS ENUM ('google', 'github', 'email');
EXCEPTION
WHEN duplicate_object THEN null;
END $$;
-- =====================
-- HELPER FUNCTION FOR UPDATED_AT
-- =====================
CREATE OR REPLACE FUNCTION trigger_set_timestamp() RETURNS TRIGGER AS $$ BEGIN NEW.updated_at = NOW();
RETURN NEW;
END;
$$ LANGUAGE plpgsql;
-- =====================
-- OAUTH USERS
-- =====================
CREATE TABLE IF NOT EXISTS oauth_users (
    id VARCHAR(255) PRIMARY KEY,
    provider provider NOT NULL,
    username VARCHAR(50),
    full_name VARCHAR(50),
    email VARCHAR(100),
    phone_number VARCHAR(50),
    password TEXT,
    picture TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT uq_oauth_email UNIQUE(email),
    CONSTRAINT uq_oauth_phone UNIQUE(phone_number)
);
-- =====================
-- USERS
-- =====================
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    full_name VARCHAR(100) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    phone_number VARCHAR(50) NOT NULL,
    password TEXT NOT NULL,
    picture TEXT,
    role user_role NOT NULL DEFAULT 'regular',
    status user_status NOT NULL DEFAULT 'active',
    email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    oauth_user_id VARCHAR(255) REFERENCES oauth_users(id) ON DELETE
    SET NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TRIGGER set_users_timestamp BEFORE
UPDATE ON users FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();
-- =====================
-- APARTMENTS
-- =====================
CREATE TABLE IF NOT EXISTS apartments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title VARCHAR(255) NOT NULL,
    description TEXT,
    rooms INTEGER NOT NULL,
    beds INTEGER NOT NULL,
    baths INTEGER NOT NULL,
    area DOUBLE PRECISION NOT NULL,
    apartment_floor INTEGER NOT NULL,
    total_building_floors INTEGER NOT NULL,
    condition apartment_condition NOT NULL,
    sale_type sale_type NOT NULL,
    requirements TEXT,
    furnished BOOLEAN NOT NULL,
    pets_allowed BOOLEAN NOT NULL,
    has_elevator BOOLEAN NOT NULL,
    has_garden BOOLEAN NOT NULL,
    has_parking BOOLEAN NOT NULL,
    has_balcony BOOLEAN NOT NULL,
    has_ac BOOLEAN NOT NULL,
    has_heating BOOLEAN NOT NULL,
    distance_to_kindergarten INTEGER NOT NULL,
    distance_to_school INTEGER NOT NULL,
    distance_to_hospital INTEGER NOT NULL,
    distance_to_metro INTEGER NOT NULL,
    distance_to_bus_stop INTEGER NOT NULL,
    distance_to_shopping INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TRIGGER set_apartments_timestamp BEFORE
UPDATE ON apartments FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();
-- =====================
-- APARTMENT AMENITIES
-- =====================
CREATE TABLE IF NOT EXISTS apartment_amenities (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    apartment_id UUID NOT NULL REFERENCES apartments(id) ON DELETE CASCADE,
    amenity VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(apartment_id, amenity)
);
CREATE TRIGGER set_apartment_amenities_timestamp BEFORE
UPDATE ON apartment_amenities FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();
-- =====================
-- APARTMENT PICTURES
-- =====================
CREATE TABLE IF NOT EXISTS apartment_pictures (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    apartment_id UUID NOT NULL REFERENCES apartments(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    is_primary BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE UNIQUE INDEX one_primary_picture_per_apartment ON apartment_pictures (apartment_id)
WHERE is_primary;
CREATE TRIGGER set_apartment_pictures_timestamp BEFORE
UPDATE ON apartment_pictures FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();
-- =====================
-- ADDRESSES
-- =====================
CREATE TABLE IF NOT EXISTS addresses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    apartment_id UUID NOT NULL UNIQUE REFERENCES apartments(id) ON DELETE CASCADE,
    street_address TEXT NOT NULL,
    city VARCHAR(100) NOT NULL,
    state_or_region VARCHAR(100) NOT NULL,
    county_or_district VARCHAR(100),
    postal_code VARCHAR(20) NOT NULL,
    country VARCHAR(100) NOT NULL,
    latitude DOUBLE PRECISION,
    longitude DOUBLE PRECISION,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TRIGGER set_addresses_timestamp BEFORE
UPDATE ON addresses FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();
-- =====================
-- LISTINGS
-- =====================
CREATE TABLE IF NOT EXISTS listings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    apartment_id UUID NOT NULL REFERENCES apartments(id) ON DELETE CASCADE,
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    price NUMERIC(12, 2) NOT NULL,
    -- currency CHAR(3) NOT NULL DEFAULT 'UZS',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TRIGGER set_listings_timestamp BEFORE
UPDATE ON listings FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();
-- =====================
-- LISTING TAGS
-- =====================
CREATE TABLE IF NOT EXISTS listing_tags (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    listing_id UUID NOT NULL REFERENCES listings(id) ON DELETE CASCADE,
    tag VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(listing_id, tag)
);
CREATE TRIGGER set_listing_tags_timestamp BEFORE
UPDATE ON listing_tags FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();
-- =====================
-- FAVORITES
-- =====================
CREATE TABLE IF NOT EXISTS favorites (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    listing_id UUID NOT NULL REFERENCES listings(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (user_id, listing_id)
);
CREATE TRIGGER set_favorites_timestamp BEFORE
UPDATE ON favorites FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();