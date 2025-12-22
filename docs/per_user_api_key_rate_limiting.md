# Per-User API Key Rate Limiting - Implementation Plan

## Overview

This document outlines the architecture and implementation for assigning unique SearXNG API keys to each licensed user, enabling per-user rate limiting.

---

## Architecture

```
┌─────────────────┐      ┌──────────────────┐      ┌─────────────────┐
│  Overlook App   │      │  License Service │      │   Redis Cloud   │
│                 │      │                  │      │                 │
│  - Validates    │─────▶│  - Creates keys  │─────▶│                 │
│    license      │      │  - Stores keys   │      │  - API keys     │
│  - Gets API key │◀─────│    in Redis      │      │  - Rate limits  │
│  - Searches     │      │                  │      │                 │
└────────┬────────┘      └──────────────────┘      └────────┬────────┘
         │                                                   │
         │                                                   │
         │               ┌──────────────────┐                │
         │               │  SearXNG Server  │                │
         └──────────────▶│                  │◀───────────────┘
                         │  - Validates key │
                         │  - Rate limits   │
                         │  - Returns results│
                         └──────────────────┘
```

**Flow:**
1. User activates license via License Service
2. License Service generates API key and stores in Redis Cloud
3. License validation response includes the API key
4. App uses API key for SearXNG requests
5. SearXNG validates key and enforces rate limits via Redis

---

## Redis Data Model

### Key Validation
```
Key:    apikey:{api_key}
Value:  {"license_id": "lic_xxx", "email": "user@example.com"}
TTL:    None (or match license expiry)
```

### Rate Limiting
```
Key:    ratelimit:{api_key}:{window}
Value:  {count}  (integer)
TTL:    Auto-expire after window (e.g., 3600 for hourly)

Example:
  ratelimit:sk_abc123:2025-12-21-14 → 42
```

### Request Flow
```
1. Check EXISTS apikey:{key}
   → Not found? Reject (401 Unauthorized)

2. INCR ratelimit:{key}:{current_hour}
   → First request in window? EXPIRE key 3600

3. If count > limit (e.g., 100/hour)
   → Reject (429 Too Many Requests)

4. Allow request → proxy to SearXNG
```

---

## Components Requiring Changes

### 1. License Service

**Files to modify:**
- `license-service/src/models.rs` - Add `searxng_api_key` field
- `license-service/src/handlers/validate.rs` - Return API key in response
- `license-service/src/paddle.rs` - Generate API key on license creation
- `license-service/src/db.rs` - Store/retrieve API key

**New dependencies:**
```toml
# Cargo.toml
redis = { version = "0.27", features = ["tokio-comp", "aio"] }
```

**Database migration:**
```sql
ALTER TABLE licenses ADD COLUMN searxng_api_key VARCHAR(64) UNIQUE;
```

**API key generation:**
```rust
fn generate_api_key() -> String {
    format!("sk_{}", uuid::Uuid::new_v4().to_string().replace("-", ""))
}
```

**Validation response update:**
```rust
pub struct ValidateResponse {
    pub valid: bool,
    pub email: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub days_remaining: Option<i64>,
    pub searxng_api_key: Option<String>,  // NEW
}
```

**Redis operations (on license creation):**
```rust
async fn store_api_key(redis: &Redis, api_key: &str, license_id: &str, email: &str) {
    let value = json!({
        "license_id": license_id,
        "email": email
    });
    redis.set(format!("apikey:{}", api_key), value.to_string()).await?;
}
```

**Redis operations (on license revocation):**
```rust
async fn revoke_api_key(redis: &Redis, api_key: &str) {
    redis.del(format!("apikey:{}", api_key)).await?;
}
```

---

### 2. SearXNG Server (Rate Limiting Proxy)

**Option A: Nginx + Lua (Recommended)**

Requires: `nginx-extras` or OpenResty for Lua support.

**Nginx configuration:**
```nginx
# /etc/nginx/sites-available/searxng

lua_shared_dict rate_limit 10m;

upstream searxng {
    server 127.0.0.1:8080;
}

server {
    listen 443 ssl;
    server_name search.yourdomain.com;

    # SSL config...

    location /search {
        access_by_lua_block {
            local redis = require "resty.redis"
            local red = redis:new()
            red:set_timeout(1000)

            -- Connect to Redis Cloud
            local ok, err = red:connect("YOUR_REDIS_CLOUD_HOST", YOUR_REDIS_CLOUD_PORT)
            if not ok then
                ngx.status = 500
                ngx.say("Redis connection failed")
                return ngx.exit(500)
            end

            -- Authenticate with Redis Cloud
            red:auth("YOUR_REDIS_CLOUD_PASSWORD")

            -- Get API key from header
            local api_key = ngx.req.get_headers()["X-API-Token"]
            if not api_key then
                ngx.status = 401
                ngx.say("Missing API key")
                return ngx.exit(401)
            end

            -- Validate API key exists
            local exists = red:exists("apikey:" .. api_key)
            if exists == 0 then
                ngx.status = 401
                ngx.say("Invalid API key")
                return ngx.exit(401)
            end

            -- Rate limiting
            local hour = os.date("%Y-%m-%d-%H")
            local rate_key = "ratelimit:" .. api_key .. ":" .. hour
            local count = red:incr(rate_key)

            -- Set TTL on first request of window
            if count == 1 then
                red:expire(rate_key, 3600)
            end

            -- Check limit (100 requests/hour)
            if count > 100 then
                ngx.status = 429
                ngx.header["Retry-After"] = "3600"
                ngx.say("Rate limit exceeded")
                return ngx.exit(429)
            end

            red:set_keepalive(10000, 100)
        }

        proxy_pass http://searxng;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

**Option B: Separate Rust Proxy Service**

If Lua is too complex, create a small Rust service that sits in front of SearXNG:

```rust
// rate-limit-proxy/src/main.rs
async fn proxy_search(
    headers: HeaderMap,
    Query(params): Query<SearchParams>,
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    let api_key = headers
        .get("X-API-Token")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate key
    let exists: bool = state.redis
        .exists(format!("apikey:{}", api_key))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !exists {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Rate limit
    let hour = chrono::Utc::now().format("%Y-%m-%d-%H");
    let rate_key = format!("ratelimit:{}:{}", api_key, hour);
    let count: i64 = state.redis.incr(&rate_key, 1).await?;

    if count == 1 {
        state.redis.expire(&rate_key, 3600).await?;
    }

    if count > 100 {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // Forward to SearXNG
    let response = state.http_client
        .get(&format!("{}/search", state.searxng_url))
        .query(&params)
        .send()
        .await?;

    Ok(response.into())
}
```

---

### 3. Overlook App (Client)

**Files to modify:**
- `src/license/mod.rs` - Store returned API key
- `src/license/storage.rs` - Persist API key locally
- `src/tools.rs` - Use per-user API key instead of env variable

**License data update:**
```rust
// src/license/storage.rs
pub struct LicenseData {
    pub license_key: Option<String>,
    pub first_launch: DateTime<Utc>,
    pub last_validation: Option<DateTime<Utc>>,
    pub cached_response: Option<ValidateResponse>,
    pub searxng_api_key: Option<String>,  // NEW
}
```

**Update after validation:**
```rust
// src/license/mod.rs
pub async fn validate_license_online(license_key: &str) -> Result<ValidateResponse> {
    let response = client.validate(license_key, &hardware_id).await?;

    // Store the API key locally
    if let Some(api_key) = &response.searxng_api_key {
        let mut data = load_license_data()?;
        data.searxng_api_key = Some(api_key.clone());
        save_license_data(&data)?;
    }

    Ok(response)
}
```

**Use in search requests:**
```rust
// src/tools.rs
impl WebSearchTool {
    pub fn new() -> Self {
        // Prefer per-user API key, fall back to env for development
        let api_token = load_license_data()
            .ok()
            .and_then(|d| d.searxng_api_key)
            .or_else(|| std::env::var("SEARXNG_API_TOKEN").ok());

        Self {
            client: reqwest::Client::new(),
            base_url: std::env::var("SEARXNG_URL").unwrap_or_default(),
            api_token,
        }
    }
}
```

---

## Implementation Steps

### Phase 1: License Service Updates
1. Add `searxng_api_key` column to database
2. Add Redis client dependency
3. Generate API key in Paddle webhook handler
4. Store API key in both PostgreSQL and Redis
5. Return API key in validation response
6. Add revocation logic to remove key from Redis

### Phase 2: SearXNG Rate Limiting
1. Install OpenResty or nginx-extras on SearXNG server
2. Configure Lua script for Redis-based rate limiting
3. Set up connection to Redis Cloud
4. Test rate limiting behavior
5. Configure appropriate limits (e.g., 100 requests/hour)

### Phase 3: Overlook App Updates
1. Update `LicenseData` struct to include API key
2. Store API key after successful validation
3. Update `WebSearchTool` to use per-user API key
4. Handle 401/429 responses gracefully in UI
5. Test end-to-end flow

---

## Redis Cloud Setup

1. Create account at https://redis.io/cloud
2. Create new free database (free tier: 30MB storage, no command limits)
3. Note connection details from the dashboard:
   - Host: `redis-xxxxx.c1.us-east-1-1.ec2.redns.redis-cloud.com`
   - Port: `xxxxx`
   - Password: `your-password`

**Environment variables for License Service:**
```bash
REDIS_CLOUD_URL=redis://default:your-password@redis-xxxxx.c1.us-east-1-1.ec2.redns.redis-cloud.com:xxxxx
```

**Environment variables for SearXNG server (nginx):**
```bash
# Or configure directly in Lua script
REDIS_HOST=redis-xxxxx.c1.us-east-1-1.ec2.redns.redis-cloud.com
REDIS_PORT=xxxxx
REDIS_PASSWORD=your-password
```

---

## Rate Limit Configuration

| Tier | Requests/Hour | Requests/Day |
|------|---------------|--------------|
| Free Trial | 20 | 100 |
| Licensed | 100 | 1000 |

Store tier info in Redis value:
```json
{
  "license_id": "lic_xxx",
  "email": "user@example.com",
  "tier": "licensed",
  "hourly_limit": 100
}
```

---

## Error Handling

### 401 Unauthorized
- Invalid or missing API key
- App should prompt user to re-validate license

### 429 Too Many Requests
- Rate limit exceeded
- App should show user-friendly message with retry time
- Include `Retry-After` header in response

### 500 Internal Server Error
- Redis connection failed
- Fail open (allow request) or fail closed based on preference

---

## Testing Checklist

- [ ] License creation generates unique API key
- [ ] API key stored in PostgreSQL and Redis
- [ ] Validation response includes API key
- [ ] App stores and uses API key for searches
- [ ] SearXNG validates API key against Redis
- [ ] Rate limiting increments counter correctly
- [ ] Rate limit exceeded returns 429
- [ ] License revocation removes key from Redis
- [ ] App handles 401/429 errors gracefully
