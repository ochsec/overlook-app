#!/bin/bash
# Deploy SearXNG to Linode using CLI
# Usage: ./deploy-to-linode.sh

set -e

# Check if cloud-init file exists
if [ ! -f "searxng-cloud-init.yml" ]; then
    echo "Error: searxng-cloud-init.yml not found!"
    echo ""
    echo "Please generate it first:"
    echo "  ./generate-cloud-init.sh"
    echo ""
    exit 1
fi

# Check if Linode CLI is installed
LINODE_CLI=$(command -v linode-cli 2>/dev/null || echo "/Users/$USER/Library/Python/3.9/bin/linode-cli")
if [ ! -x "$LINODE_CLI" ]; then
    echo "Error: Linode CLI not installed!"
    echo ""
    echo "Install with:"
    echo "  pip3 install linode-cli"
    echo ""
    echo "Then configure with your API token:"
    echo "  $LINODE_CLI configure"
    echo ""
    exit 1
fi

# Load config to get domain
if [ -f "searxng.env" ]; then
    set -a
    source searxng.env
    set +a
fi

DOMAIN=${DOMAIN:-"search.replmade.com"}

echo "=========================================="
echo "Deploying SearXNG to Linode"
echo "=========================================="
echo ""
echo "Configuration:"
echo "  Domain: $DOMAIN"
echo "  Region: Chicago, IL (us-ord)"
echo "  Plan: Nanode 1GB (g6-nanode-1)"
echo "  OS: Ubuntu 24.04 LTS"
echo ""

# Prompt for confirmation
read -p "Deploy now? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Deployment cancelled."
    exit 0
fi

# Generate a root password
ROOT_PASS=$(openssl rand -base64 24)

echo ""
echo "Creating Linode instance..."

# Create the instance (cloud-init must be base64 encoded)
USER_DATA_B64=$(base64 -i searxng-cloud-init.yml)
LINODE_ID=$("$LINODE_CLI" linodes create \
  --type g6-nanode-1 \
  --region us-ord \
  --image linode/ubuntu24.04 \
  --label searxng-prod \
  --root_pass "$ROOT_PASS" \
  --metadata.user_data "$USER_DATA_B64" \
  --json | jq -r '.[0].id')

if [ -z "$LINODE_ID" ]; then
    echo "Error: Failed to create Linode instance!"
    exit 1
fi

echo "✓ Linode created! ID: $LINODE_ID"

# Wait for IP address to be assigned
echo ""
echo "Waiting for IP address assignment..."
sleep 5

# Get IP address
IP_ADDRESS=$("$LINODE_CLI" linodes list --json | jq -r ".[] | select(.id == $LINODE_ID) | .ipv4[0]")

echo ""
echo "=========================================="
echo "Deployment Started Successfully!"
echo "=========================================="
echo ""
echo "Instance Details:"
echo "  Linode ID: $LINODE_ID"
echo "  IP Address: $IP_ADDRESS"
echo "  Label: searxng-prod"
echo "  Root Password: $ROOT_PASS"
echo ""
echo "IMPORTANT: Save the root password above!"
echo ""
echo "Cloud-init is now running (~10 minutes)..."
echo ""
echo "=========================================="
echo "Next Steps:"
echo "=========================================="
echo ""
echo "1. Configure DNS A Record:"
echo "   Name: search"
echo "   Value: $IP_ADDRESS"
echo "   TTL: 300"
echo ""
echo "2. Wait for cloud-init to complete:"
echo "   ssh -i ~/.ssh/linode_searxng root@$IP_ADDRESS"
echo "   tail -f /var/log/cloud-init-output.log"
echo ""
echo "3. Wait for DNS propagation (5-15 min):"
echo "   dig $DOMAIN"
echo ""
echo "4. Setup SSL certificate:"
echo "   ssh -i ~/.ssh/linode_searxng root@$IP_ADDRESS"
echo "   sudo /usr/local/bin/setup-ssl.sh"
echo ""
echo "5. Test your instance:"
echo "   curl 'https://$DOMAIN/search?q=test&format=json'"
echo ""
echo "=========================================="
echo "Useful Commands:"
echo "=========================================="
echo ""
echo "# SSH into server"
echo "ssh -i ~/.ssh/linode_searxng root@$IP_ADDRESS"
echo ""
echo "# View Linode details"
echo "linode-cli linodes view $LINODE_ID"
echo ""
echo "# Monitor cloud-init progress"
echo "ssh -i ~/.ssh/linode_searxng root@$IP_ADDRESS 'tail -f /var/log/cloud-init-output.log'"
echo ""
echo "# Delete instance (if needed)"
echo "linode-cli linodes delete $LINODE_ID"
echo ""
echo "=========================================="

# Save credentials to file
cat > .linode-deployment-info << EOF
# SearXNG Linode Deployment Info
# Generated: $(date)

LINODE_ID=$LINODE_ID
IP_ADDRESS=$IP_ADDRESS
ROOT_PASSWORD=$ROOT_PASS
DOMAIN=$DOMAIN

# SSH Command
ssh -i ~/.ssh/linode_searxng root@$IP_ADDRESS

# Delete Command
linode-cli linodes delete $LINODE_ID
EOF

echo "✓ Deployment info saved to: .linode-deployment-info"
echo ""
