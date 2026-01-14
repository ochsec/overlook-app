# Stripe Production Checklist for overlook.chat

## Current Status: TEST MODE ⚠️

Your site is currently using **Stripe test mode** payment links. Here's what you need to do before going live.

---

## ✅ Already Complete

- [x] Site uses Stripe payment links (not Paddle)
- [x] Terms of Service mentions Stripe as payment processor
- [x] Privacy Policy correctly references Stripe
- [x] Success page has appropriate messaging
- [x] License service webhook handler configured for Stripe
- [x] Database schema updated for Stripe (customer_id, subscription_id, session_id)

---

## 🔄 Required Before Production Launch

### 1. Create Production Stripe Payment Link

**Current (Test Mode):**
```
https://buy.stripe.com/test_4gM9AM3V7g5S9uFgf3ejK00
```

**Steps to create production link:**

1. Go to [Stripe Dashboard](https://dashboard.stripe.com) (make sure you're NOT in test mode)
2. Navigate to: **Products** → Create a new product or use existing
3. Create product:
   - Name: `Overlook License - Annual Subscription`
   - Price: `$29.00 USD / year`
   - Billing: `Recurring - yearly`
   
4. Create a Payment Link:
   - Go to **Payment Links** → **Create payment link**
   - Select your Overlook product
   - Configure:
     - Collect customer email: ✅ YES (required for license delivery)
     - Success URL: `https://overlook.chat/success.html`
     - Allow promotion codes: Optional
     - Require billing address: Optional (recommended for tax compliance)
   
5. Copy the production payment link (will look like):
   ```
   https://buy.stripe.com/live_xxxxxxxxxx
   ```

6. Replace ALL test links in your site:
   - `index.html` (3 locations - lines 118, 127, 214)
   - Any other pages with payment links

### 2. Configure Production Webhook

**Current webhook endpoint:** (configured in license-service)
```
POST /api/v1/webhook/stripe/checkout_completed
```

**Steps:**

1. Deploy your license service to production server
2. Get your production server URL (e.g., `https://api.overlook.chat`)
3. Go to [Stripe Dashboard → Webhooks](https://dashboard.stripe.com/webhooks)
4. Click **Add endpoint**
5. Configure:
   - Endpoint URL: `https://api.overlook.chat/api/v1/webhook/stripe/checkout_completed`
   - Events to send: Select `checkout.session.completed`
   - Optional: Add `customer.subscription.deleted` (for cancellations in future)
6. Copy the **Signing secret** (starts with `whsec_`)
7. Update your production `.env`:
   ```
   STRIPE_WEBHOOK_SECRET=whsec_your_production_secret
   ```
8. Test the webhook using the "Send test webhook" button in Stripe Dashboard

### 3. Update Environment Variables

**Production `.env` file should have:**

```bash
# License Server Configuration
DATABASE_URL=postgresql://user:password@your-db-host/overlook_licenses
LISTEN_ADDR=0.0.0.0:8080

# Stripe Configuration (PRODUCTION)
STRIPE_WEBHOOK_SECRET=whsec_your_production_webhook_secret

# Email Configuration (Resend.com)
RESEND_API_KEY=re_your_production_api_key
EMAIL_FROM=support@overlook.chat

# Logging
RUST_LOG=info
```

### 4. Test Production Webhook

Before going live, test with Stripe's test mode to production endpoint:

1. Use Stripe CLI to forward to your production server:
   ```bash
   stripe listen --forward-to https://api.overlook.chat/api/v1/webhook/stripe/checkout_completed
   ```

2. Trigger a test event:
   ```bash
   stripe trigger checkout.session.completed
   ```

3. Verify in logs:
   - Webhook received (200 OK)
   - License created in database
   - Email sent to customer

### 5. Test Complete Purchase Flow (Test Mode First)

1. Use your TEST payment link on the live site
2. Complete a purchase with test card: `4242 4242 4242 4242`
3. Verify:
   - [ ] Redirected to success.html
   - [ ] Webhook received in license service
   - [ ] License created in database
   - [ ] Email sent with license key
   - [ ] License key works in Overlook app

### 6. Update Site Files for Production

Replace test payment links in these files:

**index.html** - Update 3 locations:
```html
<!-- OLD (Test) -->
<a href="https://buy.stripe.com/test_4gM9AM3V7g5S9uFgf3ejK00" ...>

<!-- NEW (Production) -->
<a href="https://buy.stripe.com/YOUR_PRODUCTION_LINK" ...>
```

Locations:
- Line 118: Navigation "Buy Now" button
- Line 127: Hero section "Get Overlook" button  
- Line 214: Purchase section "Purchase Now" button

---

## 🧪 Pre-Launch Testing Checklist

Complete these tests in ORDER:

### Test 1: Stripe Test Mode → Production Server
- [ ] Test payment link works
- [ ] Webhook received
- [ ] License created
- [ ] Email sent

### Test 2: Stripe Production Mode → Test Purchase
- [ ] Production payment link works
- [ ] Can complete test purchase
- [ ] Webhook received
- [ ] License created
- [ ] Email sent
- [ ] License validates in app

### Test 3: Real Purchase (Small Amount)
- [ ] Make a real $29 purchase yourself
- [ ] Verify entire flow end-to-end
- [ ] Test license activation in app
- [ ] Verify email formatting is correct

### Test 4: Error Scenarios
- [ ] What happens if webhook fails? (Check Stripe retry logic)
- [ ] What happens if email fails? (Check database email_send_attempts)
- [ ] Test duplicate webhook (idempotency check)

---

## 📊 Monitoring & Operations

### Webhook Monitoring

Check webhook health in Stripe Dashboard:
- [Stripe Webhooks](https://dashboard.stripe.com/webhooks)
- Monitor delivery success rate
- Set up alerts for failures

### Database Monitoring

Regularly check for failed emails:
```bash
cd /Users/c/github/overlook/license-service
./check-licenses.sh
# Choose option 4: Show licenses with failed emails
```

### Email Delivery Monitoring

Check Resend dashboard for:
- Email delivery rate
- Bounce rate
- Failed sends

---

## 🚀 Go-Live Steps

When ready to launch:

1. [ ] Complete all items in "Required Before Production Launch"
2. [ ] Complete all "Pre-Launch Testing"
3. [ ] Switch payment links from test to production
4. [ ] Deploy production license service with production webhook secret
5. [ ] Monitor first few purchases closely
6. [ ] Set up Stripe Dashboard notifications
7. [ ] Test subscription renewal flow (can simulate with test mode)

---

## 📝 Post-Launch

### Week 1:
- [ ] Monitor webhook delivery success rate
- [ ] Check for any failed email deliveries
- [ ] Verify license keys work for all customers
- [ ] Respond to any customer support inquiries

### Ongoing:
- [ ] Set up monitoring alerts (consider services like Better Uptime)
- [ ] Weekly check of Stripe Dashboard for disputes/issues
- [ ] Monthly review of email delivery metrics
- [ ] Backup license database regularly

---

## 🔗 Important Links

- **Stripe Dashboard:** https://dashboard.stripe.com
- **Stripe Webhooks:** https://dashboard.stripe.com/webhooks
- **Stripe Test Cards:** https://stripe.com/docs/testing
- **Resend Dashboard:** https://resend.com/emails
- **License Service Logs:** Check your production server logs

---

## ⚠️ Common Issues & Solutions

### Issue: Webhook returns 401 Unauthorized
**Solution:** Webhook secret doesn't match. Verify `STRIPE_WEBHOOK_SECRET` in `.env` matches Stripe Dashboard.

### Issue: No email sent after purchase
**Solutions:**
- Check `RESEND_API_KEY` is set correctly
- Verify `EMAIL_FROM` domain is verified in Resend
- Check license service logs for errors
- Query database: `SELECT email_sent, email_last_error FROM licenses ORDER BY created_at DESC LIMIT 5`

### Issue: Customer didn't receive license key
**Solutions:**
1. Check spam folder
2. Query database for their email: `SELECT * FROM licenses WHERE email = 'customer@email.com'`
3. Manually resend using: `GET /api/v1/resend-email?license_key=overlook-key-XXXX-XXXX-XXXX-XXXX`

### Issue: Duplicate license created
**Should not happen** - idempotency check prevents this. If it does:
1. Check database for duplicate `stripe_session_id`
2. Review webhook logs for duplicate events
3. Verify idempotency logic in `src/handlers/webhook.rs`

---

## 📞 Support

If you encounter issues:
- Stripe Support: https://support.stripe.com
- Resend Support: https://resend.com/support
- Check webhook logs in Stripe Dashboard for detailed error messages
