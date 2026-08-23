CREATE TABLE social_connections (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    platform VARCHAR(50) NOT NULL DEFAULT 'instagram',
    platform_user_id VARCHAR(255) NOT NULL,
    handle VARCHAR(255) NOT NULL,
    access_token_encrypted BYTEA NOT NULL,
    refresh_token_encrypted BYTEA,
    token_expires_at TIMESTAMP NOT NULL,
    follower_count INTEGER NOT NULL DEFAULT 0,
    engagement_rate DOUBLE PRECISION,
    audience_demographics JSONB,
    last_synced_at TIMESTAMP,
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_social_conn_user_platform ON social_connections(user_id, platform);
CREATE INDEX idx_social_conn_user_id ON social_connections(user_id);