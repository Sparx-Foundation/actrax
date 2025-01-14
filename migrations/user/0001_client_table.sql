CREATE TABLE IF NOT EXISTS "actrax_client"
(
    id         SERIAL PRIMARY KEY,
    uid        TEXT NOT NULL,
    name       TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
