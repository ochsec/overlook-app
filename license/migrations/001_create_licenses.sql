CREATE TABLE IF NOT EXISTS licenses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    license_key TEXT UNIQUE NOT NULL,
    email TEXT NOT NULL,
    hardware_id TEXT,
    created_at TEXT NOT NULL,
    activated_at TEXT,
    revoked INTEGER DEFAULT 0,
    revoked_at TEXT,
    paddle_subscription_id TEXT,
    paddle_checkout_id TEXT
);

CREATE INDEX IF NOT EXISTS idx_license_key ON licenses(license_key);
CREATE INDEX IF NOT EXISTS idx_hardware_id ON licenses(hardware_id);
