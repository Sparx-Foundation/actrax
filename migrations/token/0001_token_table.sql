CREATE TABLE IF NOT EXISTS "refresh_tokens"
(
    token      TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    client_id    INTEGER,

    CONSTRAINT fk_client FOREIGN KEY (client_id) REFERENCES "actrax_client" (id) ON DELETE CASCADE
);