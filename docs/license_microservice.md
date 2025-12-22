# What you need to build:

In the app:
- First launch: record a timestamp (stored locally)
- On each launch: check if (now - first_launch) > 30 days AND no valid license key
- If trial expired: show "Trial expired - enter license key" modal, disable functionality
- License key input field somewhere in Settings
- Basic key validation (format check locally, or ping your server)
