# SerialRUN Website Deployment Guide

## Prerequisites

- Node.js 18+ installed
- Cloudflare account (free tier works)
- Wrangler CLI (auto-installed via npx)

## Quick Deploy

```bash
# From project root
npx wrangler pages deploy website --project-name=serialrun
```

First run will:
1. Prompt to create a new Cloudflare Pages project
2. Open browser for Cloudflare login (OAuth)
3. Ask for production branch name — **set to `master`**
4. Upload files and deploy

## Important: Production Branch

When creating the project, the production branch must be set to `master`.

If deployments show as "Preview" instead of "Production":
1. Go to Pages → serialrun → **Settings** → **Production branch**
2. Click **Rename**, change from `production` to `master`
3. Redeploy to trigger a Production deployment

## Subsequent Deploys

```bash
# Project already exists, just deploy
npx wrangler pages deploy website --project-name=serialrun

# Skip git dirty warning
npx wrangler pages deploy website --project-name=serialrun --commit-dirty=true

# Deploy to specific branch
npx wrangler pages deploy website --project-name=serialrun --branch=master
```

## URLs

After deployment:
- **Production URL**: `https://serialrun.pages.dev`
- **Custom domain**: `https://serialrun.com`
- **Version URL**: `https://<hash>.serialrun.pages.dev` (specific deployment)

Production and custom domain always point to the latest `master` branch deployment.

## Custom Domain Setup

### Domain registered on Cloudflare

1. Go to Pages → serialrun → **Custom domains**
2. Click **Set up a custom domain**
3. Enter domain (e.g., `serialrun.com`)
4. Click **Continue** → **Activate domain**
5. DNS is auto-configured (CNAME record created automatically)
6. SSL is auto-provisioned (wait 1-5 minutes)

### Add www subdomain

1. Same steps as above, enter `www.serialrun.com`
2. Both `serialrun.com` and `www.serialrun.com` will work

### Domain registered elsewhere (e.g., Alibaba Cloud)

1. Get the 2 Cloudflare NS addresses from Domain Registration
2. Go to your domain registrar, change nameservers to Cloudflare's
3. Wait for DNS propagation (can take up to 24 hours)
4. Then add custom domain in Pages as above

## Project Structure

```
website/
├── index.html          # Landing page (hero, features, download, community)
├── guide.html          # User guide (standalone page, i18n supported)
├── license.html        # BSL 1.1 license explanation (i18n supported)
├── style.css           # Global styles + responsive design
├── i18n.js             # Chinese/English translations
├── script.js           # Scroll animations
├── DEPLOY_WEB.md       # This file
├── tux.svg             # Linux Tux icon
├── wechat_pay_qr.jpg   # WeChat Pay QR code
├── screenshot_en.png   # English screenshot
└── screenshot_zh.png   # Chinese screenshot
```

## Notes

- `website/` is in `.gitignore` — not pushed to Git repos
- Deployed independently from the code repository
- Screenshots in `website/` are copies from `assets/` — update both when changing
- i18n translations are in `i18n.js` — add new keys in both `en` and `zh` objects
- Sub-pages (guide, license) have their own translation objects in `<script>` tags
- Mobile responsive CSS is in `@media` sections at the bottom of `style.css`
- Deploy guide is in `website/`, not in repo `docs/`
