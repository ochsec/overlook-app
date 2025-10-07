# Web Search Integration for MVP - Implementation Plan

## Overview

This document outlines the implementation strategy for adding web search capabilities to Overlook without requiring subscriptions or pay-as-you-go APIs.

---

## Option 1: SearXNG Self-Hosted (Recommended for Privacy)

### Pros
- Completely free, no API keys or subscriptions
- Full privacy control - no external tracking
- Can run locally on user's machine via Docker
- Aggregates results from multiple search engines
- Very lightweight (300MB Docker container)
- Works on low-spec hardware (Raspberry Pi 4+)

### Cons
- Requires Docker installation
- User must run/maintain their own instance
- Some setup complexity for non-technical users

### Implementation Steps
1. Bundle Docker Compose files with application
2. Add "Setup Search" wizard in settings
3. Check for Docker, offer to start SearXNG container
4. Connect to localhost:8080 JSON API
5. Parse results and format for LLM context

### User Experience
- One-click "Enable Web Search" button
- Automatic Docker container management
- Fallback message if Docker not available

---

## Option 2: DuckDuckGo HTML Scraping (Simplest)

### Pros
- No API key required
- No external dependencies
- Works immediately without setup
- Use existing `reqwest` dependency

### Cons
- Rate limits (undocumented, likely ~100-200 requests/day)
- Could break if DuckDuckGo changes HTML
- Against ToS (technically scraping)
- May trigger bot detection

### Implementation Steps
1. Add `scraper = "0.20"` dependency
2. Scrape https://html.duckduckgo.com/html/?q=query
3. Parse result titles, URLs, snippets
4. Format for LLM with source citations
5. Add rate limiting/backoff

### Code Example
```rust
async fn search_duckduckgo(query: &str) -> Result<Vec<SearchResult>> {
    let url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding::encode(query));
    let html = reqwest::get(&url).await?.text().await?;
    // Parse with scraper crate
}
```

---

## Option 3: Brave Search API Free Tier (Best Balance)

### Pros
- Official API with generous free tier
- 2,000 queries/month free
- No credit card required
- High quality results
- Proper JSON API

### Cons
- Requires user to get API key from Brave
- 2,000/month limit (~66 searches/day)
- Requires internet connection
- User must trust Brave with queries

### Implementation Steps
1. Add Brave Search API key field in settings
2. Use existing `reqwest` for API calls
3. Endpoint: `https://api.search.brave.com/res/v1/web/search`
4. Parse JSON response
5. Track usage (show "X/2000 searches used")

### API Call Example
```rust
reqwest::Client::new()
    .get("https://api.search.brave.com/res/v1/web/search")
    .header("X-Subscription-Token", api_key)
    .query(&[("q", query)])
    .send().await?
```

---

## Recommended Hybrid Approach for MVP

### Phase 1: Start with Brave Search API
- Easiest to implement quickly
- Good UX (just add API key)
- 2,000/month sufficient for testing
- Document as "recommended" option

### Phase 2: Add DuckDuckGo Fallback
- Use when Brave quota exhausted
- Or as "no API key" option
- Add user warning about rate limits

### Phase 3: Add SearXNG Support (Optional)
- For power users who want full privacy
- Document Docker setup in help
- Detect if SearXNG running on localhost

---

## Implementation Timeline

### Week 1: Core Infrastructure
1. Create `src/search.rs` module
2. Define `SearchResult` type with title, url, snippet, source
3. Create `SearchProvider` enum (Brave, DuckDuckGo, SearXNG)
4. Implement Brave Search API client

### Week 2: UI Integration
1. Add search API key field to settings
2. Add "Web Search" toggle when composing messages
3. Show search results as context before LLM response
4. Display search sources/citations in response

### Week 3: Enhanced Features
1. Add DuckDuckGo scraper as fallback
2. Implement rate limiting and error handling
3. Add search result caching (avoid duplicate searches)
4. Show usage stats for Brave API

### Week 4: Polish
1. Add SearXNG local detection/setup (optional)
2. Format search results nicely for LLM
3. Add "Search performed" indicator in chat
4. Documentation and user guide

---

## User Flow Example

```
User: "What's the latest news about Rust 2025?"

App Flow:
1. User enables "Web Search" checkbox
2. App searches via Brave API (or fallback)
3. Gets top 5 results with titles, URLs, snippets
4. Formats as context: "Search Results:\n1. [Title] - [Snippet] (Source: URL)\n..."
5. Sends to LLM with original query + search context
6. LLM generates response citing the sources
7. Shows "🔍 Web search performed" badge on message
```

---

## Technical Details

### Dependencies to Add
```toml
scraper = "0.20"          # For DuckDuckGo HTML parsing
urlencoding = "2.1"       # For URL encoding
serde_json = "1.0"        # Already have this
```

### New Types
```rust
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub source: SearchProvider,
}

pub enum SearchProvider {
    Brave,
    DuckDuckGo,
    SearXNG,
}
```

### Storage
- Add `brave_search_api_key: Option<String>` to settings
- Add `search_enabled: bool` to app state
- Cache results for 1 hour to avoid duplicate searches

---

## Cost Analysis

| Option | Setup Cost | Ongoing Cost | User Effort | Privacy |
|--------|-----------|--------------|-------------|---------|
| Brave API | Free | Free (2k/mo) | Get API key | Moderate |
| DuckDuckGo | Free | Free (rate limited) | None | Good |
| SearXNG | Free | Free (unlimited) | Install Docker | Excellent |

**Recommendation:** Start with Brave API for MVP, add others later.

---

## SearXNG Docker Setup (For Documentation)

### Quick Start
```bash
# Clone the official docker setup
git clone https://github.com/searxng/searxng-docker.git
cd searxng-docker

# Generate secret key
sed -i "s|ultrasecretkey|$(openssl rand -hex 32)|g" searxng/settings.yml

# Start the container
docker-compose up -d
```

### Access
Once running, SearXNG is available at `http://localhost:8080`

### API Endpoint
```
GET http://localhost:8080/search?q=<query>&format=json
```

---

## Privacy Considerations

### Brave Search API
- Queries are logged by Brave for service improvement
- User IP address visible to Brave
- Can be mitigated with VPN/proxy

### DuckDuckGo Scraping
- User IP visible to DuckDuckGo
- Queries may be logged
- Rate limiting based on IP

### SearXNG Self-Hosted
- Complete privacy - all queries stay local
- No external tracking or logging
- Best option for privacy-conscious users

---

## Error Handling

### Rate Limit Exceeded
- Show user-friendly message
- Suggest alternative provider
- Offer to enable fallback automatically

### Network Errors
- Retry with exponential backoff
- Cache last successful results
- Graceful degradation (LLM works without search)

### Invalid API Key
- Clear error message with link to get key
- Suggest DuckDuckGo fallback option

---

## Future Enhancements

1. **Search Result Ranking**
   - Allow user to re-rank results
   - Learn from which results LLM uses most

2. **Advanced Filters**
   - Time range (past day, week, month, year)
   - Site-specific search
   - File type filtering

3. **Search History**
   - Track searches per project
   - Avoid duplicate searches
   - Show search analytics

4. **Multiple Search Providers**
   - Allow user to select preferred provider
   - Automatic fallback chain
   - Aggregate results from multiple sources

5. **Custom Search Engines**
   - Add support for custom SearXNG instances
   - Allow URL input for self-hosted search

---

## Testing Strategy

### Unit Tests
- Test each search provider independently
- Mock HTTP responses
- Validate result parsing

### Integration Tests
- Test with real APIs (rate limit aware)
- Test fallback mechanisms
- Test caching behavior

### Manual Testing
- Various query types (news, technical, general)
- Rate limit handling
- Error scenarios
- UI feedback and indicators

---

## Documentation for Users

### Getting Started with Web Search

**Option 1: Brave Search (Recommended)**
1. Visit https://brave.com/search/api/
2. Sign up for free API key (no credit card)
3. Copy your API key
4. In Overlook: Settings → Web Search → Paste API key
5. Enable "Web Search" toggle when chatting

**Option 2: DuckDuckGo (No Setup)**
1. Enable "Use DuckDuckGo" in settings
2. No API key required
3. Limited to ~100 searches/day

**Option 3: SearXNG (Maximum Privacy)**
1. Install Docker Desktop
2. In Overlook: Settings → Web Search → "Setup Local Search"
3. Click "Install SearXNG"
4. Wait for Docker container to start
5. Enable "Web Search" toggle when chatting

---

# SearXNG Production Deployment on Linode VPS

## Overview

This section provides detailed instructions for deploying SearXNG on a Linode VPS to serve hundreds of users with your Overlook application. This approach gives you complete control, unlimited searches, and maximum privacy for your users.

---

## Server Requirements for Few Hundred Users

### Recommended Linode Plan: **Shared CPU 2GB**

**Specifications:**
- **CPU:** 1 vCPU (shared)
- **RAM:** 2GB
- **Storage:** 50GB SSD
- **Transfer:** 2TB
- **Cost:** ~$12/month

**Why this plan:**
- SearXNG is extremely lightweight (~300MB Docker container)
- Users report running SearXNG on 512MB RAM with no issues
- 2GB provides comfortable headroom for:
  - SearXNG application
  - Redis/Valkey for rate limiting
  - Nginx reverse proxy
  - System overhead
  - Concurrent request handling

### Scaling Considerations

| Users | Recommended Plan | CPU | RAM | Monthly Cost |
|-------|------------------|-----|-----|--------------|
| 1-100 | Shared CPU 1GB | 1 | 1GB | $5 |
| 100-500 | Shared CPU 2GB | 1 | 2GB | $12 |
| 500-2000 | Shared CPU 4GB | 2 | 4GB | $24 |
| 2000+ | Dedicated CPU 4GB | 2 | 4GB | $36 |

**Note:** Start with the 2GB plan and monitor usage. Linode allows easy upgrades if needed.

---

## Step-by-Step Deployment Guide

### Prerequisites

1. **Domain name** (e.g., `search.yourdomain.com`)
2. **Linode account**
3. **Basic Linux knowledge**

### Step 1: Create Linode Instance

```bash
# Via Linode Cloud Manager:
1. Click "Create Linode"
2. Choose Distribution: Ubuntu 24.04 LTS
3. Region: Select closest to your users
4. Plan: Shared CPU - 2GB RAM
5. Root Password: Create strong password
6. SSH Keys: Add your public key (recommended)
7. Click "Create Linode"
```

### Step 2: Initial Server Setup

```bash
# SSH into your server
ssh root@your-server-ip

# Update system
apt update && apt upgrade -y

# Install required packages
apt install -y curl git nginx certbot python3-certbot-nginx ufw

# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sh get-docker.sh

# Install Docker Compose
apt install -y docker-compose

# Enable Docker to start on boot
systemctl enable docker
systemctl start docker
```

### Step 3: Configure Firewall

```bash
# Allow SSH, HTTP, and HTTPS
ufw allow 22/tcp
ufw allow 80/tcp
ufw allow 443/tcp

# Enable firewall
ufw --force enable

# Verify status
ufw status
```

### Step 4: Deploy SearXNG with Docker

```bash
# Create directory for SearXNG
mkdir -p /opt/searxng
cd /opt/searxng

# Clone the official docker setup
git clone https://github.com/searxng/searxng-docker.git .

# Generate a secret key
SECRET_KEY=$(openssl rand -hex 32)

# Update settings
sed -i "s|ultrasecretkey|$SECRET_KEY|g" searxng/settings.yml

# Configure instance name (optional)
sed -i 's/instance_name: "SearXNG"/instance_name: "Overlook Search"/' searxng/settings.yml
```

### Step 5: Configure SearXNG for Production

Edit `/opt/searxng/searxng/settings.yml`:

```yaml
# Instance settings
general:
  instance_name: "Overlook Search"

# Enable JSON API
search:
  formats:
    - html
    - json

# Configure rate limiting
server:
  limiter: true  # Enable rate limiting
  secret_key: "YOUR_SECRET_KEY_HERE"

# Configure Redis/Valkey for rate limiting
redis:
  url: redis://redis:6379/0
```

Create `docker-compose.override.yml` for production settings:

```yaml
version: '3.7'

services:
  searxng:
    restart: always
    environment:
      - SEARXNG_HOSTNAME=search.yourdomain.com
      - SEARXNG_BASE_URL=https://search.yourdomain.com
    ports:
      - "127.0.0.1:8080:8080"

  redis:
    image: valkey/valkey:alpine
    restart: always
    command: valkey-server --save 30 1 --loglevel warning
    volumes:
      - valkey-data:/data

volumes:
  valkey-data:
```

### Step 6: Start SearXNG

```bash
cd /opt/searxng
docker-compose up -d

# Verify it's running
docker-compose ps
docker-compose logs -f searxng
```

Test locally:
```bash
curl http://localhost:8080
```

### Step 7: Configure Nginx Reverse Proxy

Create `/etc/nginx/sites-available/searxng`:

```nginx
server {
    listen 80;
    server_name search.yourdomain.com;

    # Proxy to SearXNG
    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header X-Script-Name /;

        # Important for JSON API
        proxy_set_header Accept application/json;
    }

    # Optional: Basic authentication for additional security
    # Uncomment and configure if you want password protection
    # auth_basic "Restricted";
    # auth_basic_user_file /etc/nginx/.htpasswd;
}
```

Enable the site:
```bash
ln -s /etc/nginx/sites-available/searxng /etc/nginx/sites-enabled/
nginx -t  # Test configuration
systemctl reload nginx
```

### Step 8: Configure DNS

In your domain registrar:
```
Type: A Record
Name: search
Value: YOUR_LINODE_IP
TTL: 300 (or default)
```

Wait for DNS propagation (usually 5-15 minutes).

### Step 9: Enable SSL with Let's Encrypt

```bash
# Get SSL certificate (Certbot will auto-configure Nginx)
certbot --nginx -d search.yourdomain.com

# Follow prompts:
# - Enter email for renewal notices
# - Agree to terms
# - Choose to redirect HTTP to HTTPS (recommended)

# Verify auto-renewal is configured
systemctl status certbot.timer

# Test renewal process
certbot renew --dry-run
```

Your Nginx config will be automatically updated with SSL settings.

### Step 10: Configure Rate Limiting for Hundreds of Users

Edit `/opt/searxng/searxng/limiter.toml`:

```toml
[botdetection.ip_limit]
# Filter IPs based on rate limiting
# Limits requests per IP address
window = 30  # seconds
max_requests = 30  # Allow 30 requests per 30 seconds per IP

[botdetection.ip_lists]
# Pass list - IPs that bypass rate limiting
# Add your application server IPs here
pass_ip = [
    # Add your Overlook application server IP
    # "1.2.3.4",
    # Or IP ranges:
    # "10.0.0.0/8",      # Private network
    # "192.168.0.0/16",  # Private network
]

# Block specific IPs if needed
block_ip = []
```

### Step 11: Configure Trusted Proxies

Edit `/opt/searxng/searxng/settings.yml`:

```yaml
server:
  # Trust X-Forwarded-For from Nginx
  trusted_proxies:
    - 127.0.0.1
```

Restart SearXNG:
```bash
cd /opt/searxng
docker-compose restart searxng
```

---

## Authentication Options for Production

### Option 1: API Token Authentication (Recommended)

Configure your Overlook application to send an API token with each request:

Edit Nginx config to validate tokens:

```nginx
location / {
    # Check for valid API token
    if ($http_x_api_token != "YOUR_SECRET_TOKEN") {
        return 403;
    }

    proxy_pass http://localhost:8080;
    # ... rest of proxy settings
}
```

### Option 2: IP Whitelist (Simplest)

If your Overlook application servers have static IPs:

```nginx
location / {
    # Allow only your application servers
    allow 1.2.3.4;      # Your app server IP
    allow 5.6.7.8;      # Another app server
    deny all;

    proxy_pass http://localhost:8080;
    # ... rest of proxy settings
}
```

### Option 3: HTTP Basic Auth (Quick Setup)

```bash
# Install htpasswd utility
apt install apache2-utils

# Create password file
htpasswd -c /etc/nginx/.htpasswd overlook_user

# Nginx config already includes auth_basic lines (uncommented)
nginx -t && systemctl reload nginx
```

---

## Monitoring and Maintenance

### Check SearXNG Logs

```bash
cd /opt/searxng
docker-compose logs -f searxng
docker-compose logs -f redis
```

### Monitor Resource Usage

```bash
# Check system resources
htop

# Check Docker stats
docker stats

# Check disk usage
df -h

# Check Nginx access logs
tail -f /var/log/nginx/access.log

# Check Nginx error logs
tail -f /var/log/nginx/error.log
```

### Update SearXNG

```bash
cd /opt/searxng

# Pull latest images
docker-compose pull

# Restart with new images
docker-compose up -d

# Clean up old images
docker image prune -a
```

### Backup Configuration

```bash
# Backup SearXNG settings
tar -czf searxng-backup-$(date +%Y%m%d).tar.gz /opt/searxng/searxng/

# Backup Nginx config
tar -czf nginx-backup-$(date +%Y%m%d).tar.gz /etc/nginx/sites-available/searxng
```

---

## Performance Tuning for Hundreds of Users

### Increase Nginx Worker Connections

Edit `/etc/nginx/nginx.conf`:

```nginx
events {
    worker_connections 2048;  # Increase from default 768
    multi_accept on;
    use epoll;
}

http {
    # Enable keepalive
    keepalive_timeout 65;
    keepalive_requests 100;

    # Enable compression
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_types text/plain text/css application/json;
}
```

### Optimize SearXNG Settings

Edit `/opt/searxng/searxng/settings.yml`:

```yaml
# Limit concurrent requests per engine
outgoing:
  max_request_timeout: 10.0  # Timeout after 10 seconds
  pool_connections: 100      # Connection pool size
  pool_maxsize: 20          # Max connections per engine

# Configure search engines wisely
engines:
  # Disable slow or unreliable engines
  # Keep only fast, reliable ones for production
```

### Add Caching with Redis

Already configured in docker-compose.override.yml above. Valkey/Redis caches search results.

---

## Cost Optimization

### Total Monthly Cost Breakdown

| Item | Cost |
|------|------|
| Linode 2GB VPS | $12/mo |
| Domain (if new) | $12/year (~$1/mo) |
| **Total** | **~$13/month** |

**Cost per user:**
- 100 users: $0.13/user/month
- 500 users: $0.026/user/month

**Comparison:**
- Brave Search API: $0 (2000 queries/month limit)
- SerpAPI: $50+/month for unlimited
- Custom solution: $13/month for **unlimited searches**

---

## Connecting Overlook to Your SearXNG Instance

### Configuration in Overlook

Add to settings:

```rust
pub struct SearchSettings {
    pub searxng_url: String,  // "https://search.yourdomain.com"
    pub api_token: Option<String>,  // If using token auth
}
```

### API Request Example

```rust
async fn search_searxng(query: &str, settings: &SearchSettings) -> Result<Vec<SearchResult>> {
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/search", settings.searxng_url))
        .header("X-API-Token", settings.api_token.as_deref().unwrap_or(""))
        .query(&[
            ("q", query),
            ("format", "json"),
            ("language", "en"),
            ("safesearch", "1"),
        ])
        .send()
        .await?;

    let json: serde_json::Value = response.json().await?;

    // Parse results
    let results = json["results"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|r| SearchResult {
            title: r["title"].as_str().unwrap_or("").to_string(),
            url: r["url"].as_str().unwrap_or("").to_string(),
            snippet: r["content"].as_str().unwrap_or("").to_string(),
            source: SearchProvider::SearXNG,
        })
        .collect();

    Ok(results)
}
```

---

## Troubleshooting

### Issue: "Connection Refused"

```bash
# Check if SearXNG is running
docker-compose ps

# Check logs
docker-compose logs searxng

# Restart if needed
docker-compose restart searxng
```

### Issue: "SSL Certificate Error"

```bash
# Renew certificate manually
certbot renew

# Check certificate status
certbot certificates
```

### Issue: "Rate Limiting Too Aggressive"

Edit `/opt/searxng/searxng/limiter.toml` and increase limits:

```toml
[botdetection.ip_limit]
window = 60  # Increase window
max_requests = 60  # Increase limit
```

### Issue: "Slow Search Results"

```bash
# Check which engines are slow
docker-compose logs searxng | grep -i timeout

# Disable slow engines in settings.yml
```

---

## Security Best Practices

1. **Keep system updated:**
   ```bash
   apt update && apt upgrade -y
   ```

2. **Monitor failed login attempts:**
   ```bash
   tail -f /var/log/auth.log
   ```

3. **Use fail2ban for SSH protection:**
   ```bash
   apt install fail2ban
   systemctl enable fail2ban
   ```

4. **Regular backups:**
   - Automate config backups
   - Store off-server (Linode Backups: +$2/month)

5. **Monitor disk usage:**
   - SearXNG logs can grow large
   - Configure log rotation

6. **Use environment variables for secrets:**
   - Never commit API tokens to git
   - Use `.env` files for sensitive data

---

## Summary

You now have a production-ready SearXNG instance running on Linode that can:

✅ Handle hundreds of concurrent users
✅ Provide unlimited free searches
✅ Maintain complete privacy
✅ Cost only ~$13/month
✅ Scale easily as needed
✅ Integrate seamlessly with Overlook

Your Overlook application can now offer web search as a core feature without any per-query costs or third-party API limitations.
