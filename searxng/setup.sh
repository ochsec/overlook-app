#!/bin/bash
# Manual SearXNG installation script for Linode server
# Run this on the server as root if cloud-init fails
# Usage: bash internal-install.sh

set -e

echo "=========================================="
echo "SearXNG Manual Installation"
echo "=========================================="
echo ""

# Update system
echo "Updating system packages..."
apt-get update && apt-get upgrade -y

# Install Docker
echo "Installing Docker..."
curl -fsSL https://get.docker.com -o get-docker.sh
sh get-docker.sh
systemctl enable docker
systemctl start docker
rm get-docker.sh

# Install Docker Compose
echo "Installing Docker Compose plugin..."
apt-get install -y docker-compose-plugin

# Install Nginx and Certbot
echo "Installing Nginx and Certbot..."
apt-get install -y nginx certbot python3-certbot-nginx

# Create SearXNG directory
echo "Creating SearXNG directory..."
mkdir -p /opt/searxng
cd /opt/searxng

# Create docker-compose.yml
echo "Creating docker-compose.yml..."
cat > docker-compose.yml << 'EOF'
version: '3.7'

services:
  searxng:
    image: searxng/searxng:latest
    container_name: searxng
    restart: unless-stopped
    ports:
      - "127.0.0.1:8080:8080"
    volumes:
      - ./searxng:/etc/searxng:rw
    environment:
      - SEARXNG_BASE_URL=https://search.replmade.com
    networks:
      - searxng

  redis:
    image: valkey/valkey:7-alpine
    container_name: redis
    restart: unless-stopped
    command: valkey-server --save 30 1 --loglevel warning
    networks:
      - searxng
    volumes:
      - valkey-data:/data

networks:
  searxng:

volumes:
  valkey-data:
EOF

# Start containers to generate config
echo "Starting Docker containers..."
docker compose up -d

# Wait for config generation
echo "Waiting for initial config generation..."
sleep 10

# Generate secret key
SECRET_KEY=$(openssl rand -hex 32)

# Configure SearXNG settings
echo "Configuring SearXNG settings..."
cat > searxng/settings.yml << EOF
use_default_settings: true

server:
  secret_key: "$SECRET_KEY"
  limiter: false
  image_proxy: true

outgoing:
  request_timeout: 3.0

search:
  safe_search: 0
  autocomplete: "google"
  formats:
    - html
    - json

ui:
  static_use_hash: true
  default_theme: simple
  query_in_title: true
  infinite_scroll: true

enabled_plugins:
  - 'Hash plugin'
  - 'Self Information'
  - 'Tracker URL remover'
  - 'Ahmia blacklist'

engines:
  - name: google
    disabled: false
  - name: bing
    disabled: false
  - name: duckduckgo
    disabled: false
  - name: brave
    disabled: false
EOF

# Create limiter.toml
cat > searxng/limiter.toml << 'EOF'
[botdetection.ip_limit]
# activate link_token method in the ip_limit method
link_token = true
EOF

# Restart SearXNG to apply settings
echo "Restarting SearXNG..."
docker compose restart searxng

# Configure Nginx
echo "Configuring Nginx..."
cat > /etc/nginx/sites-available/searxng << 'EOF'
server {
    listen 80;
    server_name search.replmade.com;

    location / {
        # API token validation
        if ($http_x_api_token != "a276a0f52897f633c2836e9dabee1c8284533cc80752a4fe6f066600178e02bb") {
            return 401 "Unauthorized: Invalid or missing API token\n";
        }

        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
}
EOF

# Enable site
ln -sf /etc/nginx/sites-available/searxng /etc/nginx/sites-enabled/
rm -f /etc/nginx/sites-enabled/default

# Test and restart Nginx
echo "Testing Nginx configuration..."
nginx -t
systemctl restart nginx

# Setup firewall
echo "Configuring firewall..."
ufw allow OpenSSH
ufw allow 'Nginx Full'
ufw --force enable

echo ""
echo "=========================================="
echo "Installation Complete!"
echo "=========================================="
echo ""
echo "Next steps:"
echo ""
echo "1. Configure DNS A record:"
echo "   Name: search"
echo "   Value: $(curl -s ifconfig.me)"
echo "   TTL: 300"
echo ""
echo "2. Wait for DNS propagation (5-15 minutes):"
echo "   dig search.replmade.com"
echo ""
echo "3. Setup SSL certificate:"
echo "   certbot --nginx -d search.replmade.com --non-interactive --agree-tos -m search@replmade.com"
echo ""
echo "4. Test your instance:"
echo "   curl 'https://search.replmade.com/search?q=test&format=json'"
echo ""
echo "=========================================="
