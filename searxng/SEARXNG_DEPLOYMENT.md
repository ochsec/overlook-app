# SearXNG Production Deployment

Simple deployment of SearXNG on Linode for unlimited web search with Overlook.

## Quick Start

### 1. Configure Your Environment

```bash
cd docs/

# Copy example config
cp searxng.env.example searxng.env

# Edit with your details
nano searxng.env
```

**Update these values:**
- `EMAIL` - Your email for SSL certificates
- `SSH_PUBLIC_KEY` - Your public key (from `cat ~/.ssh/linode_searxng.pub`)
- `API_TOKEN` - Generate with `openssl rand -hex 32` (or leave empty to disable auth)
- `DOMAIN` - Already set to `search.replmade.com`

### 2. Generate Cloud-Init File

```bash
./generate-cloud-init.sh
```

This creates `searxng-cloud-init.yml` with your configuration.

### 3. Deploy to Linode

**Option A: Using Linode CLI (Recommended)**

```bash
# Install Linode CLI (if not already installed)
pip3 install linode-cli

# Configure with your API token
# Note: If linode-cli is not in PATH, use full path:
# /Users/$USER/Library/Python/3.9/bin/linode-cli configure
linode-cli configure

# Deploy automatically
./deploy-to-linode.sh
```

The script will:
- Create a Linode instance in Chicago
- Apply your cloud-init configuration
- Generate and save a root password
- Display your server's IP address
- Show next steps

**Option B: Using Web Interface**

1. Go to https://cloud.linode.com
2. Click **Create Linode**
3. Select:
   - **Distribution**: Ubuntu 24.04 LTS
   - **Region**: Chicago, IL (Midwest US)
   - **Plan**: Nanode 1GB ($5/month)
   - **Label**: `searxng-prod`
4. Under **Add User Data**, paste contents of `searxng-cloud-init.yml`
5. Click **Create Linode**

### 4. Configure DNS

While server is deploying, set up DNS:

**Add A Record to replmade.com:**
- **Name**: `search`
- **Value**: [Your Linode IP - shown in dashboard]
- **TTL**: 300

Wait 5-15 minutes for DNS propagation.

### 5. Setup SSL Certificate

```bash
# SSH into your server
ssh -i ~/.ssh/linode_searxng root@YOUR_LINODE_IP

# Run SSL setup (DNS must be configured first!)
sudo /usr/local/bin/setup-ssl.sh
```

### 6. Test Your Instance

```bash
# Test without auth
curl 'https://search.replmade.com/search?q=rust&format=json'

# Test with API token (if enabled)
curl -H 'X-API-Token: YOUR_TOKEN' \
  'https://search.replmade.com/search?q=rust&format=json'
```

## Management Commands

```bash
# Monitor status
ssh root@YOUR_IP
/usr/local/bin/monitor-searxng.sh

# View logs
cd /opt/searxng
docker-compose logs -f searxng

# Update SearXNG
/usr/local/bin/update-searxng.sh

# Restart services
cd /opt/searxng
docker-compose restart
```

## Integration with Overlook

Add to your Overlook settings:

```rust
SearchSettings {
    enabled: true,
    provider: SearchProvider::SearXNG,
    searxng_url: "https://search.replmade.com".to_string(),
    api_token: Some("YOUR_API_TOKEN".to_string()), // if auth enabled
}
```

## Cost

- **Linode 1GB**: $5/month
- **SSL Certificate**: Free (Let's Encrypt)
- **Total**: $5/month for unlimited searches

## Scaling

| Users | Plan | RAM | Cost |
|-------|------|-----|------|
| 1-100 | Nanode 1GB | 1GB | $5/mo |
| 100-500 | Linode 2GB | 2GB | $12/mo |
| 500-2000 | Linode 4GB | 4GB | $24/mo |

Upgrade via Linode dashboard: Linodes → searxng-prod → Resize

## Troubleshooting

### Cloud-init still running
```bash
tail -f /var/log/cloud-init-output.log
cloud-init status
```

### SearXNG not responding
```bash
cd /opt/searxng
docker-compose ps
docker-compose restart searxng
```

### SSL certificate failed
```bash
# Verify DNS
dig search.replmade.com

# Check if it matches server IP
curl ifconfig.me

# Try again
/usr/local/bin/setup-ssl.sh
```

## Files

- `searxng.env.example` - Configuration template
- `searxng.env` - Your config (git-ignored)
- `generate-cloud-init.sh` - Generates deployment file
- `searxng-cloud-init.yml` - Generated deployment (git-ignored)

## Security

- SSH key authentication enforced
- Firewall configured (UFW)
- Fail2ban enabled for SSH
- Optional API token authentication
- SSL/TLS encryption

## Support

- SearXNG Docs: https://docs.searxng.org/
- Linode Support: https://www.linode.com/support/
- Issues: Check `/var/log/` on server
