CREATE TABLE IF NOT EXISTS "actrax_logs"
(
    id         SERIAL PRIMARY KEY,
    level      TEXT NOT NULL,
    message    TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    client_id    INTEGER,

    CONSTRAINT fk_client FOREIGN KEY (client_id) REFERENCES "actrax_client" (id) ON DELETE CASCADE
);