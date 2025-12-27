# SearXNG Rate Limiting - Implementation Plan

## Overview

This document outlines the strategy for:
1. Rate limiting SearXNG requests to prevent abuse (nginx IP-based limiting)
2. Authenticating requests with a shared API key (fetched from license service)

---

## Architecture

```
┌─────────────────┐      ┌──────────────────┐      ┌──────────────────────────────────┐
│  Overlook App   │      │  License Service │      │        SearXNG Server            │
│                 │      │                  │      │                                  │
│  1. Validate    │─────▶│  Returns API key │      │                                  │
│     license     │◀─────│                  │      │                                  │
│                 │      └──────────────────┘      │                                  │
│  2. Search      │──────────────────────────────▶│  nginx (validates key, rate      │
│     (with key)  │                               │  limits by IP)                   │
│                 │◀──────────────────────────────│         ↓                        │
│                 │                               │  SearXNG (returns results)       │
└─────────────────┘                               └──────────────────────────────────┘
```

**Flow:**
1. App launches → validates license with license service
2. License service returns validation response including SearXNG API key
3. App stores API key in memory
4. App sends search request with API key in header
5. Nginx validates API key → if invalid, return 401
6. Nginx checks request rate for client IP → if over limit, return 429
7. If valid and under limit → forward to SearXNG

---

## How It Works

Nginx's `limit_req` module uses a "leaky bucket" algorithm:
- Requests are allowed at a steady rate (e.g., 10 requests/second)
- A `burst` buffer allows short spikes above the rate
- Excess requests beyond the burst are rejected with 429
- Counters are stored in nginx shared memory (fast, no external database)
- Counters reset on nginx restart (acceptable for rate limiting)

---

## Nginx Configuration

Add to your nginx configuration:

```nginx
# In the http block (usually /etc/nginx/nginx.conf)
# Allocates 10MB shared memory zone for tracking IPs
# 10MB can track ~160,000 unique IPs
limit_req_zone $binary_remote_addr zone=search_limit:10m rate=10r/s;

# In your server block
server {
    listen 443 ssl;
    server_name search.yourdomain.com;

    # SSL config...

    location /search {
        # Validate API key
        if ($http_x_api_token != "your-secret-token-here") {
            return 401;
        }

        # Apply rate limiting
        # burst=20: allow up to 20 requests to queue during spikes
        # nodelay: don't delay burst requests, process immediately
        limit_req zone=search_limit burst=20 nodelay;

        # Return 429 instead of default 503 when rate limited
        limit_req_status 429;

        # Proxy to SearXNG
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
```

To rotate the API key, update both the nginx config and the license service, then reload nginx.

---

## Rate Limit Configuration Options

### Conservative (recommended to start)
```nginx
limit_req_zone $binary_remote_addr zone=search_limit:10m rate=10r/s;
limit_req zone=search_limit burst=20 nodelay;
```
- 10 requests/second sustained
- Burst of 20 additional requests allowed
- Suitable for normal usage patterns

### Stricter
```nginx
limit_req_zone $binary_remote_addr zone=search_limit:10m rate=1r/s;
limit_req zone=search_limit burst=5 nodelay;
```
- 1 request/second sustained
- Burst of 5 additional requests allowed
- For tighter control

### More Permissive
```nginx
limit_req_zone $binary_remote_addr zone=search_limit:10m rate=30r/s;
limit_req zone=search_limit burst=50 nodelay;
```
- 30 requests/second sustained
- Burst of 50 additional requests allowed
- For power users or if you see legitimate traffic being blocked

---

## Implementation Steps

### Phase 1: License Service
1. Add `searxng_api_key` field to `ValidateResponse`
2. Configure the API key via environment variable
3. Return API key in validation response

### Phase 2: SearXNG Server
1. SSH into SearXNG server
2. Edit nginx configuration to add `limit_req_zone` in http block
3. Add API key validation and `limit_req` directive to the search location block
4. Test configuration: `nginx -t`
5. Reload nginx: `systemctl reload nginx`
6. Test with curl (should get 401 without key, 200 with key)

### Phase 3: Overlook App
1. Update app to read API key from validation response
2. Store API key in memory (or persist locally for offline use)
3. Include API key in search request headers
4. Handle 401 responses (prompt license re-validation)

---

## Testing

Test that rate limiting works:

```bash
# Send 30 rapid requests - should see some 429 responses
for i in {1..30}; do
    curl -s -o /dev/null -w "%{http_code}\n" "https://search.yourdomain.com/search?q=test&format=json"
done
```

---

## Monitoring

Check for rate-limited requests in nginx logs:

```bash
# Count 429 responses
grep " 429 " /var/log/nginx/access.log | wc -l

# See recent rate-limited requests
grep " 429 " /var/log/nginx/access.log | tail -20
```

---

## License Service Changes

**File:** `license-service/src/models.rs`

```rust
pub struct ValidateResponse {
    pub valid: bool,
    pub email: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub days_remaining: Option<i64>,
    pub searxng_api_key: Option<String>,  // NEW
}
```

**File:** `license-service/src/handlers/validate.rs`

```rust
// Read from environment variable
let searxng_api_key = std::env::var("SEARXNG_API_KEY").ok();

// Include in response
ValidateResponse {
    valid: true,
    email: license.email,
    expires_at: license.expires_at,
    days_remaining: Some(days),
    searxng_api_key,
}
```

**Environment variable:**
```bash
SEARXNG_API_KEY=your-secret-token-here
```

---

## Overlook App Changes

**File:** `src/tools.rs`

```rust
impl WebSearchTool {
    pub fn new(searxng_api_key: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: std::env::var("SEARXNG_URL").unwrap_or_default(),
            api_token: searxng_api_key,
        }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>, SearchError> {
        let mut request = self.client
            .get(format!("{}/search", self.base_url))
            .query(&[("q", query), ("format", "json")]);

        if let Some(token) = &self.api_token {
            request = request.header("X-API-Token", token);
        }

        let response = request.send().await?;

        match response.status() {
            StatusCode::OK => { /* parse results */ }
            StatusCode::UNAUTHORIZED => Err(SearchError::Unauthorized),
            StatusCode::TOO_MANY_REQUESTS => Err(SearchError::RateLimited),
            _ => Err(SearchError::ServerError),
        }
    }
}
```

Pass the API key from the license validation response when initializing the search tool.

---

## Key Rotation

To rotate the API key:

1. Generate a new key
2. Update `SEARXNG_API_KEY` environment variable in license service
3. Restart license service
4. Update nginx config with new key
5. Reload nginx: `systemctl reload nginx`

Apps will get the new key on their next license validation (app launch).

---

## Benefits of This Approach

- **No external dependencies** - No Redis, no external cache service
- **Minimal code changes** - Small additions to license service and app
- **Fast** - Rate limiting uses nginx shared memory, no network calls
- **Resilient** - Counters rebuild automatically after restart
- **Rotatable** - API key can be changed without app updates
- **Simple** - Easy to understand and maintain
- **Battle-tested** - nginx rate limiting is widely used in production

---

## Limitations

- Rate limiting is per-IP, not per-user (acceptable since each desktop app has a unique IP)
- Users behind the same NAT/VPN share a rate limit (rare edge case)
- Single shared API key - if leaked, must rotate (but rotation is easy)
- No differentiation between trial and licensed users (could add later if needed)
- Requires app restart to get new API key after rotation
