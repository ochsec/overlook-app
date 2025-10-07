#!/bin/bash
# Generate cloud-init.yml from template with environment variables
# Usage: ./generate-cloud-init.sh

set -e

# Check if .env file exists
if [ ! -f "searxng.env" ]; then
    echo "Error: searxng.env not found!"
    echo ""
    echo "Please create searxng.env from the example:"
    echo "  cp searxng.env.example searxng.env"
    echo "  nano searxng.env"
    echo ""
    exit 1
fi

# Load environment variables (using set -a to export all variables)
set -a
source searxng.env
set +a

# Validate required variables
if [ -z "$EMAIL" ]; then
    echo "Error: EMAIL not set in searxng.env"
    exit 1
fi

if [ -z "$SSH_PUBLIC_KEY" ]; then
    echo "Error: SSH_PUBLIC_KEY not set in searxng.env"
    exit 1
fi

if [ -z "$DOMAIN" ]; then
    echo "Error: DOMAIN not set in searxng.env"
    exit 1
fi

# Set defaults if not provided
INSTANCE_NAME=${INSTANCE_NAME:-"Overlook Search"}
API_TOKEN=${API_TOKEN:-""}

echo "Generating cloud-init.yml with:"
echo "  Email: $EMAIL"
echo "  Domain: $DOMAIN"
echo "  Instance: $INSTANCE_NAME"
echo "  API Token: ${API_TOKEN:+[SET]}${API_TOKEN:-[NOT SET - auth disabled]}"
echo ""

# Generate the cloud-init file
cat > searxng-cloud-init.yml << EOF
#cloud-config
# ============================================================================
# SearXNG Production Deployment - Auto-Generated
# ============================================================================
# Generated on: $(date)
# Domain: $DOMAIN
# Instance: Linode 1GB Shared CPU (\$5/month)
#
# This file was generated from searxng-cloud-init.template.yml
# To regenerate: ./generate-cloud-init.sh
# ============================================================================

package_update: true
package_upgrade: true

packages:
  - curl
  - git
  - nginx
  - certbot
  - python3-certbot-nginx
  - ufw
  - htop
  - fail2ban
  - apache2-utils

# ============================================================================
# SSH Configuration
# Your SSH public key is automatically added for secure access
# ============================================================================
ssh_authorized_keys:
  - $SSH_PUBLIC_KEY

users:
  - name: searxng
    groups: docker
    sudo: ['ALL=(ALL) NOPASSWD:ALL']
    shell: /bin/bash
    ssh_authorized_keys:
      - $SSH_PUBLIC_KEY

# ============================================================================
# Installation Steps
# ============================================================================
runcmd:
  # Install Docker
  - curl -fsSL https://get.docker.com -o /tmp/get-docker.sh
  - sh /tmp/get-docker.sh
  - systemctl enable docker
  - systemctl start docker
  - usermod -aG docker searxng

  # Install Docker Compose
  - apt install -y docker-compose

  # Configure firewall (ports: SSH, HTTP, HTTPS)
  - ufw --force enable
  - ufw allow 22/tcp
  - ufw allow 80/tcp
  - ufw allow 443/tcp

  # Enable fail2ban for SSH brute-force protection
  - systemctl enable fail2ban
  - systemctl start fail2ban

  # Create SearXNG directory
  - mkdir -p /opt/searxng
  - cd /opt/searxng

  # Clone official SearXNG docker setup
  - git clone https://github.com/searxng/searxng-docker.git /opt/searxng
  - cd /opt/searxng

  # Generate secret key for SearXNG
  - SECRET_KEY=\$(openssl rand -hex 32)
  - sed -i "s|ultrasecretkey|\${SECRET_KEY}|g" /opt/searxng/searxng/settings.yml

  # Set instance name
  - sed -i 's/instance_name:.*$/instance_name: "$INSTANCE_NAME"/' /opt/searxng/searxng/settings.yml

  # Set ownership
  - chown -R searxng:searxng /opt/searxng

# ============================================================================
# Configuration Files
# ============================================================================
write_files:
  # Docker Compose Override - Production Settings
  - path: /opt/searxng/docker-compose.override.yml
    content: |
      version: '3.7'

      services:
        searxng:
          restart: always
          environment:
            - SEARXNG_HOSTNAME=$DOMAIN
            - SEARXNG_BASE_URL=https://$DOMAIN
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

  # Rate Limiting Configuration
  - path: /opt/searxng/searxng/limiter.toml
    content: |
      [botdetection.ip_limit]
      # Allow 30 requests per 30 seconds per IP
      window = 30
      max_requests = 30

      [botdetection.ip_lists]
      # Add your application server IPs here to bypass rate limiting
      pass_ip = [
        # Example: "1.2.3.4",
      ]
      block_ip = []

  # Nginx Reverse Proxy Configuration
  - path: /etc/nginx/sites-available/searxng
    content: |
      server {
          listen 80;
          server_name $DOMAIN;

          location / {
              proxy_pass http://localhost:8080;
              proxy_set_header Host \$host;
              proxy_set_header X-Real-IP \$remote_addr;
              proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
              proxy_set_header X-Forwarded-Proto \$scheme;
              proxy_buffering off;

$(if [ -n "$API_TOKEN" ]; then
cat << 'APITOKEN'
              # API Token Authentication
              set $token_ok 0;
              if ($http_x_api_token = "API_TOKEN_PLACEHOLDER") {
                  set $token_ok 1;
              }
              if ($token_ok = 0) {
                  return 403 "Access denied - Invalid or missing API token";
              }
APITOKEN
fi)
          }

          # Health check endpoint (no auth)
          location /healthz {
              proxy_pass http://localhost:8080;
              access_log off;
          }
      }

  # SSL Setup Script
  - path: /usr/local/bin/setup-ssl.sh
    permissions: '0755'
    content: |
      #!/bin/bash
      set -e

      EMAIL="$EMAIL"
      DOMAIN="$DOMAIN"

      echo "=== Setting up SSL for \$DOMAIN ==="

      # Verify DNS points to this server
      SERVER_IP=\$(curl -s ifconfig.me)
      DNS_IP=\$(dig +short \$DOMAIN | tail -n1)

      if [ "\$SERVER_IP" != "\$DNS_IP" ]; then
          echo "ERROR: DNS mismatch!"
          echo "Server IP: \$SERVER_IP"
          echo "DNS points to: \$DNS_IP"
          echo "Update your DNS A record and wait for propagation."
          exit 1
      fi

      echo "DNS verified. Obtaining SSL certificate..."

      certbot --nginx \\
        -d \$DOMAIN \\
        --email \$EMAIL \\
        --agree-tos \\
        --no-eff-email \\
        --redirect \\
        --non-interactive

      echo ""
      echo "=== SSL Setup Complete ==="
      echo "SearXNG is now available at: https://\$DOMAIN"
$(if [ -n "$API_TOKEN" ]; then
cat << 'TESTTOKEN'
      echo ""
      echo "Test with API token:"
      echo "  curl -H 'X-API-Token: API_TOKEN_PLACEHOLDER' 'https://$DOMAIN/search?q=test&format=json'"
TESTTOKEN
fi)

  # Monitoring Script
  - path: /usr/local/bin/monitor-searxng.sh
    permissions: '0755'
    content: |
      #!/bin/bash
      echo "=== SearXNG Status ==="
      cd /opt/searxng && docker-compose ps

      echo -e "\n=== Resource Usage ==="
      docker stats --no-stream

      echo -e "\n=== System Resources ==="
      free -h
      df -h / | tail -1

      echo -e "\n=== Recent Logs ==="
      cd /opt/searxng && docker-compose logs --tail=20 searxng

  # Update Script
  - path: /usr/local/bin/update-searxng.sh
    permissions: '0755'
    content: |
      #!/bin/bash
      set -e
      echo "=== Updating SearXNG ==="
      cd /opt/searxng
      docker-compose pull
      docker-compose up -d
      docker image prune -af
      echo "Update complete!"

  # Deployment Completion Script
  - path: /usr/local/bin/complete-setup.sh
    permissions: '0755'
    content: |
      #!/bin/bash
      set -e

      echo "=== Completing SearXNG Setup ==="
      sleep 10

      # Start SearXNG
      cd /opt/searxng
      docker-compose up -d
      sleep 15

      # Enable Nginx site
      ln -sf /etc/nginx/sites-available/searxng /etc/nginx/sites-enabled/
      rm -f /etc/nginx/sites-enabled/default
      nginx -t && systemctl reload nginx

      echo ""
      echo "========================================="
      echo "SearXNG Deployment Complete!"
      echo "========================================="
      echo ""
      echo "Server IP: \$(curl -s ifconfig.me)"
      echo "Domain: $DOMAIN"
      echo ""
      echo "NEXT STEPS:"
      echo ""
      echo "1. Configure DNS A record:"
      echo "   Name: search"
      echo "   Value: \$(curl -s ifconfig.me)"
      echo "   TTL: 300"
      echo ""
      echo "2. Wait 5-15 minutes for DNS propagation"
      echo "   Check: dig $DOMAIN"
      echo ""
      echo "3. Setup SSL certificate:"
      echo "   ssh root@\$(curl -s ifconfig.me)"
      echo "   sudo /usr/local/bin/setup-ssl.sh"
      echo ""
$(if [ -n "$API_TOKEN" ]; then
cat << 'SETUPAPI'
      echo "4. API authentication is ENABLED"
      echo "   Token: API_TOKEN_PLACEHOLDER"
      echo ""
SETUPAPI
else
cat << 'NOAPI'
      echo "4. API authentication is DISABLED"
      echo "   Anyone can access your search instance"
      echo "   To enable: edit /etc/nginx/sites-available/searxng"
      echo ""
NOAPI
fi)
      echo "Monitor: /usr/local/bin/monitor-searxng.sh"
      echo "Update: /usr/local/bin/update-searxng.sh"
      echo "========================================="

# ============================================================================
# Run Setup
# ============================================================================
runcmd:
  - /usr/local/bin/complete-setup.sh

# ============================================================================
# Completion Message
# ============================================================================
final_message: |
  SearXNG deployment initiated!
  Check /var/log/cloud-init-output.log for progress.
  Run: tail -f /var/log/cloud-init-output.log
EOF

# Replace API token placeholder if set
if [ -n "$API_TOKEN" ]; then
    sed -i.bak "s/API_TOKEN_PLACEHOLDER/$API_TOKEN/g" searxng-cloud-init.yml
    rm searxng-cloud-init.yml.bak
fi

echo "✓ Generated: searxng-cloud-init.yml"
echo ""
echo "Next steps:"
echo "1. Create Linode instance"
echo "2. Paste contents of searxng-cloud-init.yml into 'Add User Data'"
echo "3. Wait ~10 minutes for deployment"
echo "4. Configure DNS"
echo "5. Run setup-ssl.sh"
echo ""
echo "To view the generated file:"
echo "  cat searxng-cloud-init.yml"
