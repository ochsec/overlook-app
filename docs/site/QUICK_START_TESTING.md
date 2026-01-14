# Quick Start: Testing Stripe Integration

This guide gets you testing the Stripe integration in **under 5 minutes**.

## Prerequisites

- Overlook site deployed (or running locally)
- License service running locally (see license-service/TEST_WEBHOOK.md)
- Stripe CLI installed (`brew install stripe/stripe-cli/stripe`)

---

## 5-Minute Test Flow

### Step 1: Start License Service (Terminal 1)

```bash
cd /Users/c/github/overlook/license-service
cargo run
```

Wait for: `License server ready`

### Step 2: Forward Webhooks (Terminal 2)

```bash
stripe listen --forward-to localhost:8080/api/v1/webhook/stripe/checkout_completed
```

**Important:** Copy the webhook signing secret and update `.env`:
```bash
# In /Users/c/github/overlook/license-service/.env
STRIPE_WEBHOOK_SECRET=whsec_xxxxxxxxxxxxx  # paste the secret here
```

Restart Terminal 1 (cargo run)

### Step 3: Open the Site

```bash
# If running locally:
open /Users/c/github/overlook/overlook-site/index.html

# Or if deployed:
open https://overlook.chat
```

### Step 4: Make a Test Purchase

1. Click "Buy Now - $29/year" button
2. You'll be redirected to Stripe checkout (test mode)
3. Fill in test payment:
   - Email: `your-email@example.com` (use your real email to receive the license)
   - Card: `4242 4242 4242 4242`
   - Expiry: Any future date (e.g., `12/34`)
   - CVC: Any 3 digits (e.g., `123`)
4. Click "Subscribe"

### Step 5: Watch the Magic ✨

**In Terminal 2 (Stripe CLI):**
```
[200] POST /api/v1/webhook/stripe/checkout_completed
```

**In Terminal 1 (License Service):**
```
INFO overlook_license_server: Received Stripe webhook event_id=evt_xxxxx
INFO overlook_license_server: Generated new license key license_key=overlook-key-1A2B-3C4D-5E6F-7890
INFO overlook_license_server: Created new license
INFO overlook_license_server: License email sent successfully
```

**In Your Email:**
You should receive an email with your license key!

### Step 6: Verify in Database

```bash
cd /Users/c/github/overlook/license-service
./check-licenses.sh
# Choose option 1: Show recent licenses
```

---

## What to Test

### ✅ Happy Path (Above)
- Purchase completes successfully
- Webhook received
- License created
- Email sent

### ✅ Idempotency Test

Trigger the same event twice:

```bash
# Terminal 3
stripe trigger checkout.session.completed
stripe trigger checkout.session.completed  # Same event again
```

**Expected:** First creates license, second logs "Session already processed, skipping"

### ✅ Failed Payment Test

Use a test card that fails:
- Card: `4000 0000 0000 0002` (card declined)

**Expected:** Checkout fails, no webhook sent, no license created

### ✅ Email Failure Test

1. Set invalid Resend API key in `.env`:
   ```
   RESEND_API_KEY=invalid_key
   ```
2. Restart license service
3. Make a test purchase
4. Check database:
   ```bash
   ./check-licenses.sh
   # Choose option 4: Show licenses with failed emails
   ```

**Expected:** License created but email marked as failed

---

## Troubleshooting

### "Webhook returns 401 Unauthorized"

**Fix:** Webhook secret doesn't match
```bash
# Make sure .env has the correct secret from Terminal 2
grep STRIPE_WEBHOOK_SECRET .env
# Should match the whsec_ secret from stripe listen output
```

### "No email received"

**Checks:**
1. Check spam folder
2. Verify `RESEND_API_KEY` is set: `grep RESEND_API_KEY .env`
3. Check license service logs for email errors
4. Query database to see email status:
   ```sql
   SELECT license_key, email, email_sent, email_last_error 
   FROM licenses 
   ORDER BY created_at DESC 
   LIMIT 1;
   ```

### "License service not starting"

**Checks:**
1. Database running? `psql $DATABASE_URL -c "SELECT 1;"`
2. Port 8080 already in use? `lsof -i :8080`
3. Check `.env` file exists and is valid

---

## Quick Commands Reference

```bash
# Start license service
cd /Users/c/github/overlook/license-service && cargo run

# Forward webhooks
stripe listen --forward-to localhost:8080/api/v1/webhook/stripe/checkout_completed

# Trigger test event
stripe trigger checkout.session.completed

# Check recent licenses
cd /Users/c/github/overlook/license-service && ./check-licenses.sh

# View Stripe events
stripe events list --limit 10

# Test webhook manually
./test-webhook.sh
```

---

## Next Steps

Once basic testing works:

1. Review [STRIPE_PRODUCTION_CHECKLIST.md](./STRIPE_PRODUCTION_CHECKLIST.md)
2. Set up production webhook endpoint
3. Create production Stripe payment link
4. Test with real (small) purchase
5. Launch! 🚀
