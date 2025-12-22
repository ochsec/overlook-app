# Overlook License Server

A lightweight license validation microservice for the Overlook desktop application. Handles license key validation, device binding, and Paddle webhook integration.

## Features

- License key validation with single-device binding
- Paddle Billing webhook integration for automatic license provisioning
- SQLite storage (single file, no separate database server)
- Hardware fingerprint verification for device locking

## Quick Start

```bash
# Copy environment template
cp .env.example .env

# Edit .env with your settings
# DATABASE_URL=sqlite:./licenses.db
# LISTEN_ADDR=0.0.0.0:8080
# PADDLE_WEBHOOK_SECRET=your_paddle_secret

# Run the server
cargo run --release
```

## API Endpoints

### Health Check

```
GET /api/v1/health
```

Response:
```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

### Validate License

```
POST /api/v1/validate
Content-Type: application/json

{
  "license_key": "XXXX-XXXX-XXXX-XXXX",
  "hardware_id": "sha256:abc123..."
}
```

Success response:
```json
{
  "valid": true,
  "email": "user@example.com"
}
```

Error responses:
- `404` - License not found
- `409` - License already activated on another device
- `403` - License has been revoked

### Paddle Webhook

```
POST /api/v1/webhook/paddle
Paddle-Signature: ts=1234567890;h1=signature...
Content-Type: application/json

{ ...paddle webhook payload... }
```

Supported events:
- `transaction.completed` - Creates new license
- `subscription.created` - Creates new license
- `subscription.canceled` - Revokes license

## Database Schema

```sql
CREATE TABLE licenses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    license_key TEXT UNIQUE NOT NULL,
    email TEXT NOT NULL,
    hardware_id TEXT,              -- NULL until first activation
    created_at TEXT NOT NULL,      -- ISO 8601 timestamp
    activated_at TEXT,             -- When hardware_id was bound
    revoked INTEGER DEFAULT 0,
    revoked_at TEXT,
    paddle_subscription_id TEXT,
    paddle_checkout_id TEXT
);
```

## Configuration

Environment variables (can be set in `.env` file):

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | SQLite database path | `sqlite:./licenses.db` |
| `LISTEN_ADDR` | Server bind address | `0.0.0.0:8080` |
| `PADDLE_WEBHOOK_SECRET` | Paddle webhook signing secret | (empty) |
| `RUST_LOG` | Log level | `info` |

## Paddle Integration

### Setup

1. In Paddle dashboard, go to **Developer Tools > Notifications**
2. Add webhook destination: `https://your-domain.com/api/v1/webhook/paddle`
3. Subscribe to events: `transaction.completed`, `subscription.created`, `subscription.canceled`
4. Copy the webhook signing secret to `PADDLE_WEBHOOK_SECRET`

### License Key in Custom Data

When creating a Paddle checkout, include the license key in custom data:

```javascript
Paddle.Checkout.open({
  items: [{ priceId: 'pri_xxx', quantity: 1 }],
  customData: {
    license_key: 'XXXX-XXXX-XXXX-XXXX'  // Pre-generated or UUID
  }
});
```

The webhook handler extracts `license_key` from `custom_data` and creates the license record.

## Deployment

### Systemd Service

Create `/etc/systemd/system/overlook-license.service`:

```ini
[Unit]
Description=Overlook License Server
After=network.target

[Service]
Type=simple
User=overlook
Group=overlook
WorkingDirectory=/opt/overlook-license
ExecStart=/opt/overlook-license/overlook-license-server
Restart=always
RestartSec=5
Environment=RUST_LOG=info
Environment=DATABASE_URL=/var/lib/overlook-license/licenses.db
Environment=PADDLE_WEBHOOK_SECRET=your_secret_here
Environment=LISTEN_ADDR=0.0.0.0:8080

[Install]
WantedBy=multi-user.target
```

Setup commands:

```bash
# Create user and directories
sudo useradd -r -s /bin/false overlook
sudo mkdir -p /opt/overlook-license /var/lib/overlook-license
sudo chown overlook:overlook /var/lib/overlook-license

# Build and deploy
cargo build --release
sudo cp target/release/overlook-license-server /opt/overlook-license/

# Enable and start
sudo systemctl enable overlook-license
sudo systemctl start overlook-license
sudo systemctl status overlook-license
```

### Nginx Reverse Proxy (HTTPS)

```nginx
server {
    listen 443 ssl http2;
    server_name license.yourdomain.com;

    ssl_certificate /etc/letsencrypt/live/license.yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/license.yourdomain.com/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

# Redirect HTTP to HTTPS
server {
    listen 80;
    server_name license.yourdomain.com;
    return 301 https://$server_name$request_uri;
}
```

## Device Binding

The license server implements single-device binding:

1. **First activation**: When a license key is validated with a `hardware_id` for the first time, that hardware ID is permanently bound to the license.

2. **Subsequent validations**: If the same license key is validated from a different `hardware_id`, the request is rejected with a `409 Conflict` error.

3. **Hardware ID format**: The Overlook client generates hardware IDs as `sha256:<hash>` where the hash is derived from:
   - Linux: `/etc/machine-id`
   - macOS: `IOPlatformUUID`
   - Windows: `MachineGuid` from Registry

## Manual License Management

### Add a license manually

```bash
sqlite3 /var/lib/overlook-license/licenses.db

INSERT INTO licenses (license_key, email, created_at)
VALUES ('XXXX-XXXX-XXXX-XXXX', 'user@example.com', datetime('now'));
```

### Revoke a license

```bash
sqlite3 /var/lib/overlook-license/licenses.db

UPDATE licenses
SET revoked = 1, revoked_at = datetime('now')
WHERE license_key = 'XXXX-XXXX-XXXX-XXXX';
```

### Reset device binding

```bash
sqlite3 /var/lib/overlook-license/licenses.db

UPDATE licenses
SET hardware_id = NULL, activated_at = NULL
WHERE license_key = 'XXXX-XXXX-XXXX-XXXX';
```

## Client Configuration

The Overlook desktop app connects to the license server. Set the server URL via environment variable:

```bash
LICENSE_SERVER_URL=https://license.yourdomain.com ./overlook
```

Default: `https://license.overlook.app`
