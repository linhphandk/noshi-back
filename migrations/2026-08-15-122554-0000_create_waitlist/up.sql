CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE waitlist (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) NOT NULL UNIQUE,
    position SERIAL,
    profile_complete_at TIMESTAMP NULL,
    is_featured BOOLEAN NOT NULL DEFAULT FALSE,
    signed_up_at TIMESTAMP NOT NULL DEFAULT NOW()
);
