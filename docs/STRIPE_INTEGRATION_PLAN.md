# Stripe Integration Plan for Overlook

**Date:** December 31, 2025  
**Purpose:** Migrate from Paddle to Stripe for $29/year subscription payment processing  
**Approach:** Stripe Payment Links (no backend required on site)

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Files to Modify](#files-to-modify)
4. [Files to Create](#files-to-create)
5. [Detailed Code Changes](#detailed-code-changes)
6. [Stripe Dashboard Setup](#stripe-dashboard-setup)
7. [Webhook Integration](#webhook-integration)
8. [Email Templates](#email-templates)
9. [Testing Procedure](#testing-procedure)
10. [Migration Timeline](#migration-timeline)
11. [Important Notes](#important-notes)

---

## Overview

### Current State
- **Payment Processor:** Paddle (sandbox environment)
- **Price:** $29/year subscription
- **Implementation:** Client-side only (no backend)
- **Paddle Elements:** Script loaded from CDN, checkout overlay

### Target State
- **Payment Processor:** Stripe
- **Price:** $29/year subscription (same)
- **Implementation:** Stripe Payment Links (hosted checkout)
- **Backend:** None required on site (webhooks go to existing microservice)
- **Downloads:** Remain public and free
- **License Delivery:** Via webhook to microservice → email to customer

### Why Stripe Payment Links?

This is the simplest approach:
- No backend code needed on your site
- Stripe handles PCI compliance
- Hosted checkout page (Stripe's UI)
- Easy to test and maintain
- Minimal code changes

**Trade-off:** Less UI customization (checkout happens on Stripe's page instead of overlay)

---

## Architecture

```
User clicks "Buy Now" 
    ↓
Redirects to Stripe-hosted checkout page
    ↓
User enters payment info and completes purchase
    ↓
Stripe processes payment
    ↓
User redirected to success.html on your site
    ↓
Stripe sends webhook to your microservice
    ↓
Your microservice:
  - Generates license key
  - Sends email with license key
  - Stores subscription data
```

**Email Flow:**
1. Stripe sends receipt email (automatically)
2. Your microservice sends license key email (via webhook)
3. Customer receives both emails

---

## Files to Modify

### 1. `index.html`
- Remove Paddle SDK script and initialization
- Remove `openCheckout()` function
- Update 3 purchase buttons with Stripe Payment Link URL

### 2. `terms.html`
- Change "Paddle" to "Stripe" (1 reference at line 88)

### 3. `privacy.html`
- Update payment processor references (2 sections)
- Update links to Stripe privacy policy

---

## Files to Create

### 4. `success.html`
- New post-purchase thank you page
- Instructions to check email for license key
- Download buttons
- Support information

---

## Detailed Code Changes

### Change 1: index.html - Remove Paddle SDK (lines 8-29)

**DELETE:**
```html
<script src="https://cdn.paddle.com/paddle/v2/paddle.js"></script>
<script>
    // Wait for Paddle to be ready before initializing
    if (window.Paddle) {
        Paddle.Initialize({
            token: 'test_ae88f443c1a041bf4f47c0b1282',
            environment: 'sandbox'
        });
    } else {
        window.addEventListener('load', function() {
            Paddle.Initialize({
                token: 'test_ae88f443c1a041bf4f47c0b1282',
                environment: 'sandbox'
            });
        });
    }

    function openCheckout() {
        Paddle.Checkout.open({
            items: [{ priceId: 'pri_01kd1fkefzv1569jdsccw8rw4c' }]
        });
    }

    // OS and Architecture Detection
    function detectOS() {
```

**REPLACE WITH:**
```html
<script>
    // OS and Architecture Detection
    function detectOS() {
```

### Change 2: index.html - Update header button (line 140)

**FROM:**
```html
<a href="#purchase" class="btn btn-primary">Buy Now - $29/year</a>
```

**TO:**
```html
<a href="STRIPE_PAYMENT_LINK_URL" class="btn btn-primary">Buy Now - $29/year</a>
```

### Change 3: index.html - Update hero button (line 149)

**FROM:**
```html
<a href="#purchase" class="btn btn-primary btn-large">Get Overlook - $29/year</a>
```

**TO:**
```html
<a href="STRIPE_PAYMENT_LINK_URL" class="btn btn-primary btn-large">Get Overlook - $29/year</a>
```

### Change 4: index.html - Update purchase section button (line 236)

**FROM:**
```html
<button onclick="openCheckout()" class="btn btn-primary btn-large">Purchase Now</button>
```

**TO:**
```html
<a href="STRIPE_PAYMENT_LINK_URL" class="btn btn-primary btn-large">Purchase Now</a>
```

### Change 5: terms.html - Update payment processor (line 88)

**FROM:**
```html
<p>Overlook is offered as an annual subscription at $29 per year. Payment is processed securely through Paddle.</p>
```

**TO:**
```html
<p>Overlook is offered as an annual subscription at $29 per year. Payment is processed securely through Stripe.</p>
```

### Change 6: privacy.html - Update section 2.3 (lines 81-87)

**FROM:**
```html
<h3>2.3 Payment Information</h3>
<p>All payment processing is handled by <a href="https://www.paddle.com" target="_blank" rel="noopener noreferrer">Paddle</a>, our payment processor. We never see or store your:</p>
<ul>
    <li>Credit card numbers</li>
    <li>Billing addresses</li>
    <li>Payment details</li>
</ul>
<p>Paddle collects and processes this information according to their own <a href="https://www.paddle.com/legal/privacy" target="_blank" rel="noopener noreferrer">Privacy Policy</a>. Paddle may share basic transaction information with us (such as your email and purchase confirmation) to enable license delivery.</p>
```

**TO:**
```html
<h3>2.3 Payment Information</h3>
<p>All payment processing is handled by <a href="https://stripe.com" target="_blank" rel="noopener noreferrer">Stripe</a>, our payment processor. We never see or store your:</p>
<ul>
    <li>Credit card numbers</li>
    <li>Billing addresses</li>
    <li>Payment details</li>
</ul>
<p>Stripe collects and processes this information according to their own <a href="https://stripe.com/privacy" target="_blank" rel="noopener noreferrer">Privacy Policy</a>. Stripe may share basic transaction information with us (such as your email and purchase confirmation) to enable license delivery.</p>
```

### Change 7: privacy.html - Update section 4.2 (lines 116-117)

**FROM:**
```html
<h3>4.2 Paddle (Payment Processor)</h3>
<p>As mentioned above, Paddle processes all payments and may collect billing information, payment details, and transaction history. Please review <a href="https://www.paddle.com/legal/privacy" target="_blank" rel="noopener noreferrer">Paddle's Privacy Policy</a> for details.</p>
```

**TO:**
```html
<h3>4.2 Stripe (Payment Processor)</h3>
<p>As mentioned above, Stripe processes all payments and may collect billing information, payment details, and transaction history. Please review <a href="https://stripe.com/privacy" target="_blank" rel="noopener noreferrer">Stripe's Privacy Policy</a> for details.</p>
```

### Change 8: Create success.html (NEW FILE)

See "Files to Create" section below for full file content.

---

## Stripe Dashboard Setup

**IMPORTANT:** Complete these steps BEFORE modifying code. You'll need the Payment Link URL to insert into the code.

### Step 1: Create Product

1. Login to **Stripe Dashboard** (https://dashboard.stripe.com)
2. Toggle to **Test Mode** (switch in top-right corner)
3. Navigate to **Products** → **Add Product**
4. Fill in product details:
   - **Name:** `Overlook Annual Subscription`
   - **Description:** `Annual license for Overlook desktop AI client - includes all features, continuous updates, and support for macOS, Windows, and Linux`
   - **Pricing model:** Select `Recurring`
   - **Price:** `$29.00`
   - **Currency:** `USD`
   - **Billing period:** Select `Yearly`
5. Click **Save product**

### Step 2: Create Payment Link

1. Navigate to **Payment Links** → **New**
2. Select the "Overlook Annual Subscription" product you just created
3. Configure Payment Link settings:
   - **After payment:** Select `Redirect to success page`
   - **Success URL:** `https://overlook.chat/success.html`
   - **Cancel URL:** `https://overlook.chat/index.html#purchase`
   - **Collect customer billing address:** `Yes` (recommended)
   - **Collect customer phone number:** `Optional` (your choice)
   - **Allow promotion codes:** `Yes` (optional - enables discount codes)
4. Click **Create link**
5. **COPY THE PAYMENT LINK URL** 
   - Format: `https://buy.stripe.com/test_xxxxxxxxxxxxx`
   - Save this URL - you'll need it for code changes
   - This is the test mode link

### Step 3: Configure Webhook

1. Navigate to **Developers** → **Webhooks**
2. Click **Add endpoint**
3. Configure webhook:
   - **Endpoint URL:** `https://your-microservice-domain.com/webhook/stripe`
     - Replace with your actual microservice webhook URL
   - **Description:** `Overlook license key webhook`
   - **Events to send:** Click **Select events**
     - Select these events:
       - ✓ `checkout.session.completed`
       - ✓ `customer.subscription.created`
       - ✓ `customer.subscription.updated`
       - ✓ `customer.subscription.deleted`
       - ✓ `invoice.payment_succeeded`
       - ✓ `invoice.payment_failed`
4. Click **Add endpoint**
5. **COPY THE SIGNING SECRET**
   - Click on the webhook you just created
   - Click **Reveal** next to "Signing secret"
   - Copy the secret (starts with `whsec_...`)
   - **Provide this to your microservice** for webhook verification

### Step 4: Enable Stripe Receipt Emails

1. Navigate to **Settings** → **Emails**
2. Find **Successful payments** section
3. Toggle **ON** to enable receipt emails
4. (Optional) Customize email template:
   - Click **Customize**
   - Add Overlook branding/logo
   - Include support contact: `support@overlook.chat`
   - Add note: "Your license key will arrive in a separate email"
   - Preview and save

### Step 5: (Optional) Enable Stripe Tax

If you have international customers or need tax collection:

1. Navigate to **Settings** → **Tax**
2. Click **Enable Stripe Tax**
3. Configure tax settings for your business location
4. Stripe will automatically calculate and collect appropriate taxes

**Note:** This is especially important for EU customers (VAT) and US customers (sales tax).

---

## Webhook Integration

### Webhook Endpoint Requirements

Your microservice needs to accept POST requests at the webhook URL configured in Stripe.

**Example endpoint:** `https://your-microservice.com/webhook/stripe`

### Webhook Payload Structure

When a customer completes checkout, Stripe sends this payload:

**Event Type:** `checkout.session.completed`

```json
{
  "id": "evt_1234567890abcdef",
  "object": "event",
  "api_version": "2023-10-16",
  "created": 1704067200,
  "type": "checkout.session.completed",
  "data": {
    "object": {
      "id": "cs_test_1234567890abcdef",
      "object": "checkout.session",
      "customer": "cus_1234567890abcdef",
      "customer_email": "customer@example.com",
      "customer_details": {
        "email": "customer@example.com",
        "name": "John Doe",
        "phone": null,
        "tax_exempt": "none"
      },
      "subscription": "sub_1234567890abcdef",
      "amount_total": 2900,
      "currency": "usd",
      "payment_status": "paid",
      "status": "complete",
      "mode": "subscription"
    }
  }
}
```

### Key Fields to Extract

| Field | Purpose |
|-------|---------|
| `data.object.id` | Checkout Session ID - use for idempotency |
| `data.object.customer_email` | Email address to send license key |
| `data.object.customer_details.name` | Customer name for personalized email |
| `data.object.customer` | Stripe Customer ID - store for future reference |
| `data.object.subscription` | Subscription ID - store with license key |
| `data.object.amount_total` | Amount paid in cents (2900 = $29.00) |
| `data.object.payment_status` | Should be "paid" |

### Microservice Implementation Requirements

Your microservice webhook handler must:

#### 1. Verify Webhook Signature

**CRITICAL:** Always verify the webhook signature to ensure the request is from Stripe.

```javascript
// Example: Node.js with Express
const stripe = require('stripe')('sk_test_...');
const endpointSecret = 'whsec_...'; // Your signing secret

app.post('/webhook/stripe', express.raw({type: 'application/json'}), (req, res) => {
  const sig = req.headers['stripe-signature'];
  let event;

  try {
    event = stripe.webhooks.constructEvent(req.body, sig, endpointSecret);
  } catch (err) {
    console.error('Webhook signature verification failed:', err.message);
    return res.status(400).send(`Webhook Error: ${err.message}`);
  }

  // Process verified event
  if (event.type === 'checkout.session.completed') {
    handleCheckoutComplete(event.data.object);
  }

  res.status(200).send();
});
```

```python
# Example: Python with Flask
import stripe
from flask import Flask, request

endpoint_secret = 'whsec_...'

@app.route('/webhook/stripe', methods=['POST'])
def webhook():
    payload = request.data
    sig_header = request.headers.get('Stripe-Signature')

    try:
        event = stripe.Webhook.construct_event(
            payload, sig_header, endpoint_secret
        )
    except ValueError as e:
        return 'Invalid payload', 400
    except stripe.error.SignatureVerificationError as e:
        return 'Invalid signature', 400

    if event['type'] == 'checkout.session.completed':
        handle_checkout_complete(event['data']['object'])

    return '', 200
```

#### 2. Implement Idempotency

Stripe may send the same webhook multiple times. Store processed session IDs to avoid duplicate processing.

```javascript
// Pseudocode
const processedSessions = new Set(); // In production, use database

function handleCheckoutComplete(session) {
  const sessionId = session.id;
  
  // Check if already processed
  if (processedSessions.has(sessionId)) {
    console.log('Session already processed:', sessionId);
    return; // Skip duplicate
  }
  
  // Process the session
  const licenseKey = generateLicenseKey();
  sendLicenseEmail(session.customer_email, session.customer_details.name, licenseKey);
  
  // Mark as processed
  processedSessions.add(sessionId);
  
  // Store in database
  saveToDatabase({
    sessionId: sessionId,
    customerId: session.customer,
    subscriptionId: session.subscription,
    email: session.customer_email,
    licenseKey: licenseKey,
    createdAt: new Date()
  });
}
```

#### 3. Generate License Key

Create a unique license key for the customer.

```javascript
// Example implementation
function generateLicenseKey() {
  const crypto = require('crypto');
  const prefix = 'OVL'; // Overlook prefix
  const random = crypto.randomBytes(12).toString('hex').toUpperCase();
  
  // Format: OVL-XXXX-XXXX-XXXX-XXXX-XXXX-XXXX
  const formatted = random.match(/.{1,4}/g).join('-');
  return `${prefix}-${formatted}`;
}

// Example output: OVL-A3F2-B8C1-D4E9-F7A6-B2C8-D1E5
```

#### 4. Send License Key Email

Email the license key to the customer.

```javascript
// Example using SendGrid
const sgMail = require('@sendgrid/mail');
sgMail.setApiKey(process.env.SENDGRID_API_KEY);

async function sendLicenseEmail(email, name, licenseKey) {
  const msg = {
    to: email,
    from: 'support@overlook.chat',
    subject: 'Your Overlook License Key',
    text: `Hi ${name},\n\nYour license key: ${licenseKey}\n\n...`,
    html: generateLicenseEmailHTML(name, licenseKey)
  };
  
  await sgMail.send(msg);
  console.log('License email sent to:', email);
}
```

#### 5. Store Subscription Data

Store the subscription information for future reference.

```javascript
// Database schema example
{
  sessionId: 'cs_test_...',
  customerId: 'cus_...',
  subscriptionId: 'sub_...',
  email: 'customer@example.com',
  name: 'John Doe',
  licenseKey: 'OVL-A3F2-B8C1-...',
  status: 'active',
  createdAt: '2025-12-31T12:00:00Z',
  expiresAt: '2026-12-31T12:00:00Z'
}
```

#### 6. Return 200 OK Response

Always return a 200 status code to acknowledge receipt.

```javascript
// After processing
res.status(200).send();
```

**Important:** If you return a non-200 response, Stripe will retry the webhook (up to 3 days).

### Handling Other Webhook Events

#### `customer.subscription.updated`

Triggered when subscription renews or is modified.

```javascript
if (event.type === 'customer.subscription.updated') {
  const subscription = event.data.object;
  
  // Update subscription status in database
  updateSubscription({
    subscriptionId: subscription.id,
    status: subscription.status,
    currentPeriodEnd: new Date(subscription.current_period_end * 1000)
  });
}
```

#### `customer.subscription.deleted`

Triggered when subscription is cancelled.

```javascript
if (event.type === 'customer.subscription.deleted') {
  const subscription = event.data.object;
  
  // Mark license as expired
  updateLicenseStatus({
    subscriptionId: subscription.id,
    status: 'cancelled',
    cancelledAt: new Date()
  });
  
  // Optionally send cancellation email
  sendCancellationEmail(subscription.customer);
}
```

#### `invoice.payment_failed`

Triggered when subscription renewal payment fails.

```javascript
if (event.type === 'invoice.payment_failed') {
  const invoice = event.data.object;
  
  // Send payment failure notification
  sendPaymentFailureEmail({
    email: invoice.customer_email,
    amount: invoice.amount_due / 100,
    invoiceUrl: invoice.hosted_invoice_url
  });
}
```

---

## Email Templates

### Email 1: License Key (Sent by Your Microservice)

**From:** support@overlook.chat  
**Subject:** Your Overlook License Key  
**Trigger:** `checkout.session.completed` webhook

**Text Version:**
```
Hi [Customer Name],

Thank you for purchasing Overlook!

Your License Key: [LICENSE-KEY-HERE]

Getting Started:
1. Download Overlook for your platform: https://overlook.chat/#download
2. Install the application
3. On first launch, enter your license key when prompted

Your subscription includes:
✓ All features
✓ Continuous updates for 1 year
✓ Support for macOS, Windows, and Linux
✓ Email support at support@overlook.chat

Need help? Reply to this email or contact us at support@overlook.chat

Best regards,
The Overlook Team

---
Your subscription will automatically renew on [RENEWAL_DATE] for $29/year.
You can manage your subscription at any time by contacting support@overlook.chat
```

**HTML Version:**
```html
<!DOCTYPE html>
<html>
<head>
  <style>
    body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }
    .container { max-width: 600px; margin: 0 auto; padding: 20px; }
    .header { background: #1a2421; color: #e8e4d9; padding: 20px; text-align: center; }
    .content { padding: 30px; background: #f5f5f5; }
    .license-key { 
      background: #fff; 
      border: 2px solid #c9a227; 
      padding: 20px; 
      text-align: center; 
      font-size: 20px; 
      font-weight: bold; 
      font-family: monospace;
      margin: 20px 0;
    }
    .steps { background: #fff; padding: 20px; margin: 20px 0; }
    .steps ol { padding-left: 20px; }
    .steps li { margin-bottom: 10px; }
    .footer { text-align: center; color: #666; font-size: 12px; padding: 20px; }
  </style>
</head>
<body>
  <div class="container">
    <div class="header">
      <h1>Welcome to Overlook!</h1>
    </div>
    
    <div class="content">
      <p>Hi [Customer Name],</p>
      
      <p>Thank you for purchasing Overlook. Here's your license key:</p>
      
      <div class="license-key">
        [LICENSE-KEY-HERE]
      </div>
      
      <div class="steps">
        <h2>Getting Started</h2>
        <ol>
          <li>Download Overlook for your platform: <a href="https://overlook.chat/#download">overlook.chat/#download</a></li>
          <li>Install the application</li>
          <li>On first launch, enter your license key when prompted</li>
        </ol>
      </div>
      
      <h3>Your subscription includes:</h3>
      <ul>
        <li>✓ All features</li>
        <li>✓ Continuous updates for 1 year</li>
        <li>✓ Support for macOS, Windows, and Linux</li>
        <li>✓ Email support at support@overlook.chat</li>
      </ul>
      
      <p>Need help? Reply to this email or contact us at <a href="mailto:support@overlook.chat">support@overlook.chat</a></p>
      
      <p>Best regards,<br>The Overlook Team</p>
    </div>
    
    <div class="footer">
      <p>Your subscription will automatically renew on [RENEWAL_DATE] for $29/year.</p>
      <p>You can manage your subscription by contacting support@overlook.chat</p>
    </div>
  </div>
</body>
</html>
```

### Email 2: Receipt (Sent Automatically by Stripe)

Stripe automatically sends receipt emails when you enable them in Settings → Emails.

**Default Content Includes:**
- Receipt number
- Date of purchase
- Amount charged ($29.00)
- Payment method (last 4 digits of card)
- Billing details
- Link to invoice

**Customization Options:**
- Add your logo
- Customize colors to match brand
- Add custom message
- Include support email

**To customize:**
1. Go to Settings → Emails → Successful payments
2. Click "Customize"
3. Add your customizations
4. Preview and save

---

## Testing Procedure

### Prerequisites

- [ ] Stripe account created and verified
- [ ] Test mode enabled in Stripe Dashboard
- [ ] Product and Payment Link created in test mode
- [ ] Webhook endpoint configured in test mode
- [ ] Webhook signing secret provided to microservice
- [ ] Microservice ready to receive webhooks
- [ ] Code changes completed (use test Payment Link URL)
- [ ] Changes deployed to test/staging environment

### Test 1: Successful Payment Flow

**Objective:** Verify complete purchase-to-license-delivery flow

**Steps:**
1. Navigate to your test site
2. Click any "Buy Now" button
3. Verify redirect to Stripe checkout page
4. Fill in test payment details:
   - **Email:** `youremail+test1@example.com`
   - **Card Number:** `4242 4242 4242 4242`
   - **Expiry:** Any future date (e.g., `12/28`)
   - **CVC:** Any 3 digits (e.g., `123`)
   - **Name:** `Test Customer 1`
   - **Country:** United States
   - **ZIP:** Any 5 digits (e.g., `12345`)
5. Click "Subscribe"
6. Wait for processing
7. Verify redirect to `success.html`

**Expected Results:**
- ✓ Checkout page loads correctly
- ✓ Payment processes successfully
- ✓ Redirects to success page
- ✓ Success page displays properly
- ✓ Stripe receipt email received at test email
- ✓ License key email received at test email (from microservice)
- ✓ Both emails arrive within 1-2 minutes

**Verify in Stripe Dashboard:**
1. Go to Payments → All payments
2. Find the test payment
3. Verify status: "Succeeded"
4. Verify amount: $29.00

**Verify Webhook Delivery:**
1. Go to Developers → Webhooks
2. Click on your webhook endpoint
3. Find the latest `checkout.session.completed` event
4. Verify response code: `200`
5. Check response body/logs

**Verify in Your Microservice:**
1. Check logs for webhook receipt
2. Verify license key was generated
3. Verify email was sent
4. Check database for stored subscription

### Test 2: Payment Declined

**Objective:** Verify proper handling of failed payments

**Steps:**
1. Navigate to your test site
2. Click "Buy Now"
3. Fill in test payment details:
   - **Email:** `youremail+test2@example.com`
   - **Card Number:** `4000 0000 0000 0002` (declined card)
   - **Expiry:** Any future date
   - **CVC:** Any 3 digits
   - **Name:** `Test Customer 2`
5. Click "Subscribe"

**Expected Results:**
- ✓ Error message displays: "Your card was declined"
- ✓ User remains on checkout page
- ✓ No redirect to success page
- ✓ No emails sent
- ✓ No webhook fired
- ✓ No payment recorded in Stripe Dashboard

### Test 3: Abandoned Checkout

**Objective:** Verify cancel flow

**Steps:**
1. Navigate to your test site
2. Click "Buy Now"
3. Partially fill in checkout form
4. Click browser back button or close tab

**Expected Results:**
- ✓ Redirects to cancel URL: `index.html#purchase`
- ✓ No emails sent
- ✓ No webhook fired
- ✓ No payment recorded

### Test 4: Additional Test Cards

Test various scenarios with Stripe test cards:

| Card Number | Scenario |
|-------------|----------|
| `4242 4242 4242 4242` | Successful payment |
| `4000 0000 0000 0002` | Card declined |
| `4000 0000 0000 9995` | Insufficient funds |
| `4000 0000 0000 0069` | Expired card |
| `4000 0000 0000 0127` | Incorrect CVC |
| `4000 0025 0000 3155` | 3D Secure authentication required |

**Reference:** https://stripe.com/docs/testing

### Test 5: Webhook Retry

**Objective:** Verify webhook retry logic and idempotency

**Steps:**
1. Complete a successful test purchase
2. In Stripe Dashboard, go to Developers → Webhooks
3. Find the `checkout.session.completed` event
4. Click "Resend" to manually retry the webhook
5. Check your microservice logs

**Expected Results:**
- ✓ Webhook received again
- ✓ Idempotency check passes
- ✓ No duplicate license key generated
- ✓ No duplicate email sent
- ✓ Logs show "already processed" message
- ✓ Returns 200 OK

### Test 6: Email Delivery

**Objective:** Verify both emails are received

**Steps:**
1. Complete successful test purchase with real email address
2. Check inbox for both emails

**Expected Results:**
- ✓ Stripe receipt arrives (from Stripe)
- ✓ License key email arrives (from your microservice)
- ✓ Both emails have correct branding
- ✓ Links in emails work correctly
- ✓ License key is properly formatted
- ✓ No emails go to spam

### Test 7: Mobile Checkout

**Objective:** Verify mobile experience

**Steps:**
1. Open site on mobile device (or use browser dev tools mobile view)
2. Click "Buy Now"
3. Complete checkout on mobile

**Expected Results:**
- ✓ Checkout page is mobile-responsive
- ✓ Form is easy to fill on mobile
- ✓ Success page displays correctly on mobile

### Test 8: International Cards

**Objective:** Verify international payment support

**Test Cards:**
- `4000 0056 0000 0008` - Visa (Debit, GB)
- `4000 0025 0000 1001` - Visa (Credit, FR)
- `4000 0038 0000 0008` - Visa (Debit, CA)

**Steps:**
1. Complete checkout with international test card
2. Verify successful payment

**Expected Results:**
- ✓ Payment processes in USD
- ✓ No currency conversion issues
- ✓ Email delivered internationally

---

## Migration Timeline

### Phase 1: Preparation (Before Launch)

**Estimated Time:** 2-4 hours

- [ ] **Stripe Account Setup**
  - [ ] Create/verify Stripe account
  - [ ] Enable test mode
  - [ ] Complete business profile

- [ ] **Stripe Product Configuration**
  - [ ] Create product: "Overlook Annual Subscription"
  - [ ] Set price: $29/year recurring
  - [ ] Create Payment Link (test mode)
  - [ ] Copy test Payment Link URL

- [ ] **Webhook Setup**
  - [ ] Add webhook endpoint in Stripe Dashboard
  - [ ] Configure webhook events
  - [ ] Copy webhook signing secret
  - [ ] Provide signing secret to microservice team

- [ ] **Microservice Preparation**
  - [ ] Implement webhook handler
  - [ ] Add signature verification
  - [ ] Implement idempotency
  - [ ] Test license key generation
  - [ ] Test email sending
  - [ ] Deploy to staging/test environment

- [ ] **Email Configuration**
  - [ ] Enable Stripe receipt emails
  - [ ] Customize receipt email template
  - [ ] Test license key email template
  - [ ] Verify email deliverability

### Phase 2: Code Implementation

**Estimated Time:** 1-2 hours

- [ ] **Update index.html**
  - [ ] Remove Paddle SDK script
  - [ ] Remove Paddle initialization
  - [ ] Remove openCheckout() function
  - [ ] Update header "Buy Now" button with Stripe link
  - [ ] Update hero "Get Overlook" button with Stripe link
  - [ ] Update purchase section button with Stripe link

- [ ] **Update terms.html**
  - [ ] Change "Paddle" to "Stripe" (line 88)

- [ ] **Update privacy.html**
  - [ ] Update section 2.3 payment processor references
  - [ ] Update section 4.2 payment processor references
  - [ ] Update privacy policy links

- [ ] **Create success.html**
  - [ ] Design success page
  - [ ] Add thank you message
  - [ ] Add instructions to check email
  - [ ] Add download buttons
  - [ ] Add support information

### Phase 3: Testing

**Estimated Time:** 2-3 hours

- [ ] **Deploy to Staging**
  - [ ] Deploy code changes to staging environment
  - [ ] Verify all pages load correctly
  - [ ] Verify all buttons link to correct Stripe checkout

- [ ] **Execute Test Plan**
  - [ ] Test 1: Successful payment flow ✓
  - [ ] Test 2: Payment declined ✓
  - [ ] Test 3: Abandoned checkout ✓
  - [ ] Test 4: Additional test cards ✓
  - [ ] Test 5: Webhook retry and idempotency ✓
  - [ ] Test 6: Email delivery (both emails) ✓
  - [ ] Test 7: Mobile checkout ✓
  - [ ] Test 8: International cards ✓

- [ ] **Fix Any Issues**
  - [ ] Address bugs found during testing
  - [ ] Retest failed scenarios
  - [ ] Verify all tests pass

### Phase 4: Production Setup

**Estimated Time:** 1 hour

- [ ] **Stripe Production Mode**
  - [ ] Switch to Live mode in Stripe Dashboard
  - [ ] Create production product (same config as test)
  - [ ] Create production Payment Link
  - [ ] Copy production Payment Link URL
  - [ ] Create production webhook endpoint (same URL)
  - [ ] Configure same webhook events
  - [ ] Copy production webhook signing secret

- [ ] **Update Microservice**
  - [ ] Add production webhook signing secret to environment
  - [ ] Verify production environment variables
  - [ ] Test production webhook endpoint is accessible

- [ ] **Update Code**
  - [ ] Replace test Payment Link with production link in code
  - [ ] Verify all references updated
  - [ ] Review all changes one final time

### Phase 5: Launch

**Estimated Time:** 30 minutes + monitoring

- [ ] **Pre-Launch Checklist**
  - [ ] All tests passing
  - [ ] Production Payment Link configured
  - [ ] Production webhook configured
  - [ ] Microservice ready for production traffic
  - [ ] Email templates reviewed and approved
  - [ ] Support team briefed on new flow

- [ ] **Deploy to Production**
  - [ ] Deploy updated code to production
  - [ ] Verify site loads correctly
  - [ ] Verify all buttons link to production Stripe checkout

- [ ] **Smoke Test**
  - [ ] Make real purchase with own credit card
  - [ ] Verify complete flow works
  - [ ] Verify both emails arrive
  - [ ] Verify license key works in application
  - [ ] Test refund process in Stripe Dashboard (refund own purchase)

- [ ] **Monitor**
  - [ ] Watch for incoming purchases
  - [ ] Monitor webhook delivery (Stripe Dashboard)
  - [ ] Monitor email delivery (check logs)
  - [ ] Monitor for any customer issues

### Phase 6: Post-Launch

**Estimated Time:** Ongoing

- [ ] **First 24 Hours**
  - [ ] Monitor all purchases closely
  - [ ] Verify webhook delivery for each purchase
  - [ ] Verify email delivery for each purchase
  - [ ] Respond quickly to any customer issues

- [ ] **First Week**
  - [ ] Check webhook delivery success rate
  - [ ] Review email deliverability
  - [ ] Gather customer feedback
  - [ ] Address any issues found

- [ ] **Cleanup**
  - [ ] Remove Paddle account (after transition complete)
  - [ ] Update any documentation
  - [ ] Archive old Paddle test data

---

## Important Notes

### Paddle vs Stripe Differences

#### 1. Tax Handling

**Paddle:**
- Acts as Merchant of Record
- Automatically handles VAT/sales tax globally
- Files tax returns on your behalf
- Collects and remits taxes

**Stripe:**
- You are the Merchant of Record
- You're responsible for tax compliance
- Stripe Tax (optional add-on) can calculate and collect taxes
- You still need to file tax returns

**Recommendation:**
- For international customers (especially EU), enable **Stripe Tax**
- Go to Settings → Tax → Enable Stripe Tax
- Configure your business location
- Stripe will automatically calculate appropriate taxes at checkout

#### 2. Receipts

**Paddle:**
- Sends receipts with Paddle branding
- Limited customization

**Stripe:**
- Fully customizable receipt emails
- Your own branding
- Control over email content

**Action Required:**
- Already configured in Step 4 of Stripe Dashboard Setup
- Verify receipt emails are enabled and customized

#### 3. Checkout Experience

**Paddle:**
- Overlay checkout on your site
- Stays on your domain
- Embedded experience

**Stripe Payment Links:**
- Hosted checkout on Stripe's domain (buy.stripe.com)
- Users leave your site temporarily
- Simpler to implement (no backend needed)

**Trade-off:**
- Less customization, but much simpler implementation
- Most customers are comfortable with Stripe-hosted checkout

**Alternative (Future):**
- Can implement Stripe Checkout embedded mode later if needed
- Requires minimal backend code

#### 4. Subscription Management

**Paddle:**
- Built-in customer portal
- Customers can manage subscriptions themselves

**Stripe:**
- Customer Portal available but requires setup
- Can generate portal links via API

**Current Implementation:**
- Not included in initial launch
- Customers contact support@overlook.chat for changes

**Future Enhancement:**
- Add "Manage Subscription" link to website
- Generate Stripe Customer Portal session via serverless function
- No complex backend needed

### Optional Future Enhancements

These features can be added later without major changes:

#### 1. Stripe Customer Portal

**What it does:**
- Allows customers to:
  - Update payment method
  - Cancel subscription
  - View invoices
  - Download receipts

**How to add:**
1. Create serverless function (Netlify/Vercel/AWS Lambda)
2. Function generates Customer Portal session
3. Add "Manage Subscription" link to website
4. Link redirects to Customer Portal

**Complexity:** Low (1-2 hours)

#### 2. Stripe Tax

**What it does:**
- Automatically calculates tax based on customer location
- Collects appropriate VAT/sales tax
- Provides tax reporting

**How to enable:**
1. Go to Stripe Dashboard → Settings → Tax
2. Click "Enable Stripe Tax"
3. Configure business location
4. Tax automatically added to checkout

**Cost:** 0.5% of transaction + tax calculation fees

**Complexity:** Very low (15 minutes)

#### 3. Discount Codes

**What it does:**
- Create promotional discount codes
- Offer launch discounts, affiliate commissions, etc.

**How to add:**
1. Go to Stripe Dashboard → Products → Coupons
2. Create coupon (e.g., "LAUNCH20" for 20% off)
3. Customers enter code at checkout
4. Discount automatically applied

**Complexity:** Very low (no code changes needed)

#### 4. Multiple Pricing Options

**What it does:**
- Offer monthly subscription ($3.99/month)
- Offer lifetime license ($199 one-time)
- Let customers choose

**How to add:**
1. Create additional products in Stripe
2. Create Payment Links for each
3. Add pricing options to website
4. Each button links to different Payment Link

**Complexity:** Low (1 hour for UI changes)

#### 5. Free Trial

**What it does:**
- 7-day or 14-day free trial
- No payment until trial ends
- Automatic conversion to paid

**How to add:**
1. Edit product in Stripe Dashboard
2. Add trial period to subscription
3. Update Payment Link

**Complexity:** Very low (15 minutes)

**Note:** May require license key activation changes in app

### Security Best Practices

#### 1. Webhook Signature Verification

**CRITICAL:** Always verify webhook signatures

**Why:**
- Prevents malicious actors from sending fake webhooks
- Ensures webhook is actually from Stripe
- Protects against unauthorized license generation

**How:**
- Use Stripe SDK's built-in verification
- Never skip this step, even in development

**Example:**
```javascript
// ✓ GOOD - Verifies signature
const event = stripe.webhooks.constructEvent(body, signature, secret);

// ✗ BAD - Trusts request without verification
const event = JSON.parse(request.body);
```

#### 2. Use HTTPS for Webhooks

**Requirement:**
- Webhook endpoint MUST use HTTPS
- Stripe won't send webhooks to HTTP endpoints

**Verify:**
- Your webhook URL starts with `https://`
- SSL certificate is valid
- No mixed content warnings

#### 3. Idempotency

**Why:**
- Stripe may send the same webhook multiple times
- Network issues can cause duplicates
- Retries happen automatically

**How:**
- Store processed session IDs
- Check before processing each webhook
- Use database transactions

**Example:**
```javascript
if (await isAlreadyProcessed(sessionId)) {
  return 200; // Already handled
}

await processCheckout(session);
await markAsProcessed(sessionId);
```

#### 4. API Key Security

**Best Practices:**
- Never commit API keys to version control
- Use environment variables
- Separate test and production keys
- Rotate keys periodically

**Key Types:**
- `pk_test_...` - Publishable key (test) - can be public
- `pk_live_...` - Publishable key (live) - can be public
- `sk_test_...` - Secret key (test) - KEEP SECRET
- `sk_live_...` - Secret key (live) - KEEP SECRET
- `whsec_...` - Webhook secret - KEEP SECRET

**For this integration:**
- No secret keys needed in website code
- Only Payment Link URL needed (public)
- Webhook secret only in microservice (private)

#### 5. Rate Limiting

**Protect webhook endpoint:**
- Add rate limiting to prevent abuse
- Allow for normal Stripe retry behavior
- Monitor for unusual traffic

#### 6. Logging and Monitoring

**Log these events:**
- Webhook received (with event type)
- Signature verification success/failure
- License key generation
- Email sent success/failure
- Any errors or exceptions

**Do NOT log:**
- Full webhook payload (may contain PII)
- Customer payment details
- Full credit card numbers (never sent in webhooks anyway)

**Monitoring:**
- Set up alerts for webhook failures
- Monitor email delivery rates
- Track license generation success rate

### Stripe Dashboard Monitoring

**Key Metrics to Watch:**

1. **Payments → All payments**
   - Monitor successful payments
   - Check for failed payments
   - Review refund requests

2. **Billing → Subscriptions**
   - Active subscriptions count
   - Upcoming renewals
   - Cancelled subscriptions
   - Churn rate

3. **Developers → Webhooks**
   - Webhook delivery success rate
   - Failed webhook attempts
   - Response times

4. **Reports → Overview**
   - Revenue trends
   - Customer growth
   - Payment failure rates

### Common Issues and Solutions

#### Issue: Webhook not received

**Possible Causes:**
- Webhook endpoint URL incorrect
- Endpoint not accessible from internet
- Firewall blocking Stripe IPs
- SSL certificate invalid

**Solutions:**
1. Verify endpoint URL in Stripe Dashboard
2. Test endpoint with curl: `curl https://your-endpoint.com/webhook/stripe`
3. Check firewall rules
4. Verify SSL certificate is valid

**Debug:**
- Check Stripe Dashboard → Webhooks → Event log
- Look for failed delivery attempts
- Check error messages

#### Issue: Duplicate license keys sent

**Possible Causes:**
- Idempotency not implemented
- Webhook retried by Stripe
- Race condition in processing

**Solutions:**
1. Implement proper idempotency checks
2. Use database transactions
3. Store processed session IDs before processing

#### Issue: Emails not delivered

**Possible Causes:**
- Email service down
- Invalid email address
- Emails going to spam
- Email service API key invalid

**Solutions:**
1. Check email service logs
2. Verify API keys
3. Test with known-good email address
4. Check spam folder
5. Verify SPF/DKIM records configured

**Prevention:**
- Use reputable email service (SendGrid, Mailgun, SES)
- Configure SPF/DKIM/DMARC records
- Warm up email sending domain
- Monitor email delivery rates

#### Issue: Payment Link not working

**Possible Causes:**
- Link is for test mode but Stripe in live mode
- Link expired or deleted
- Product or price archived

**Solutions:**
1. Verify link is for correct mode (test vs live)
2. Generate new Payment Link
3. Check product status in Dashboard

#### Issue: Checkout shows wrong price

**Possible Causes:**
- Using wrong Payment Link
- Price changed but link not updated
- Currency mismatch

**Solutions:**
1. Verify Payment Link points to correct product
2. Check price in Stripe Dashboard
3. Generate new Payment Link if needed

### Support and Documentation

**Stripe Resources:**
- **Documentation:** https://stripe.com/docs
- **API Reference:** https://stripe.com/docs/api
- **Support:** https://support.stripe.com
- **Status Page:** https://status.stripe.com

**Webhook Testing:**
- **Stripe CLI:** https://stripe.com/docs/stripe-cli
  - Install CLI to test webhooks locally
  - Forward webhooks to localhost
  - Trigger test events

**Test Cards:**
- **Full List:** https://stripe.com/docs/testing
- **International Cards:** https://stripe.com/docs/testing#international-cards
- **3D Secure:** https://stripe.com/docs/testing#regulatory-cards

**Email Best Practices:**
- **SendGrid Docs:** https://docs.sendgrid.com
- **Email Deliverability:** https://sendgrid.com/blog/deliverability-guide/

### Rollback Plan

If critical issues occur after launch:

**Immediate Rollback (Emergency):**
1. Revert code to previous version (with Paddle)
2. Restore from git: `git revert HEAD`
3. Redeploy immediately
4. Notify customers of temporary payment system issues

**Partial Rollback:**
1. Keep Stripe integrated
2. Disable Payment Links in Stripe Dashboard
3. Debug issues
4. Re-enable when fixed

**Contact Affected Customers:**
- Email all customers who attempted purchase during issue
- Offer direct assistance
- Provide alternative payment method if needed

### Going Live Checklist

Before switching to production:

**Stripe Configuration:**
- [ ] Live mode enabled
- [ ] Production product created
- [ ] Production Payment Link created
- [ ] Production webhook endpoint created
- [ ] Webhook signing secret updated in microservice
- [ ] Receipt emails enabled and customized
- [ ] Stripe Tax enabled (if required)

**Code:**
- [ ] All Paddle references removed
- [ ] Production Payment Link URL inserted
- [ ] All pages tested
- [ ] Success page works correctly
- [ ] Links all point to production URLs

**Microservice:**
- [ ] Production webhook secret configured
- [ ] Production email API keys configured
- [ ] Production database configured
- [ ] Logging enabled
- [ ] Monitoring enabled
- [ ] Error alerting configured

**Testing:**
- [ ] All test scenarios passed
- [ ] Mobile tested
- [ ] Emails delivered successfully
- [ ] Webhooks delivered successfully
- [ ] License keys generated correctly

**Documentation:**
- [ ] Support team briefed
- [ ] Refund process documented
- [ ] Known issues documented
- [ ] Customer communication prepared

**Monitoring:**
- [ ] Stripe Dashboard access confirmed
- [ ] Webhook monitoring enabled
- [ ] Email delivery monitoring enabled
- [ ] Error alerts configured
- [ ] On-call person designated

**Backup Plan:**
- [ ] Rollback procedure documented
- [ ] Previous version tagged in git
- [ ] Emergency contact list ready

---

## Summary

### What's Changing

**From:**
- Paddle payment processing
- Client-side checkout overlay
- Paddle-hosted payment page

**To:**
- Stripe payment processing
- Stripe-hosted checkout page
- Webhook integration to microservice
- License delivery via email

### What's Staying the Same

- $29/year pricing
- Annual subscription model
- Public, free downloads
- Overall site structure
- User experience (minor flow change)

### Files Modified

1. `index.html` - Remove Paddle, add Stripe Payment Link (4 changes)
2. `terms.html` - Update payment processor reference (1 change)
3. `privacy.html` - Update payment processor references (2 changes)

### Files Created

1. `success.html` - Post-purchase thank you page (new file)

### External Changes Required

1. **Stripe Dashboard:**
   - Create product
   - Create Payment Link
   - Configure webhook
   - Enable receipt emails

2. **Microservice:**
   - Implement webhook handler
   - Add signature verification
   - Implement license key generation
   - Configure email sending

### Complexity Assessment

**Overall Complexity:** Low to Medium

**Time Estimate:**
- Stripe setup: 2-4 hours
- Code changes: 1-2 hours
- Testing: 2-3 hours
- **Total: 5-9 hours**

**Maintenance:** Very low (Stripe handles most complexity)

### Benefits

- No backend to maintain on site
- Stripe handles PCI compliance
- Professional checkout experience
- Easy to test and debug
- Scalable infrastructure
- Lower fees than Paddle (in most cases)

### Risks

- Tax handling responsibility shifts to you (enable Stripe Tax)
- Customers leave your domain for checkout (industry standard)
- Need reliable webhook endpoint
- Email delivery dependency

### Success Criteria

- [ ] Customers can successfully purchase subscriptions
- [ ] Customers receive receipt email from Stripe
- [ ] Customers receive license key email from microservice
- [ ] Webhooks deliver reliably (>99% success rate)
- [ ] No duplicate license keys generated
- [ ] Support volume remains manageable
- [ ] No payment processing downtime

---

## Next Steps

1. **Review this plan** with your team
2. **Set up Stripe account** and complete dashboard configuration
3. **Prepare microservice** to handle webhooks
4. **Implement code changes** (can start before microservice is ready)
5. **Test thoroughly** in test mode
6. **Deploy to staging** and verify
7. **Switch to production** when ready
8. **Monitor closely** during launch
9. **Gather feedback** and iterate

---

## Questions?

If you have questions during implementation:

**Stripe Support:**
- Email: support@stripe.com
- Chat: Available in Stripe Dashboard
- Docs: https://stripe.com/docs

**Implementation Questions:**
- Contact support@overlook.chat
- Refer to this document
- Check Stripe documentation

---

**Document Version:** 1.0  
**Last Updated:** December 31, 2025  
**Author:** Overlook Development Team
