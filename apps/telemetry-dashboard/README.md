# Wisp Telemetry Dashboard

Read-only administration UI for the existing `wisp-telemetry` D1 database and
`wisp-telemetry-errors` R2 bucket. It does not share routes or code with the public telemetry
collector.

## Local development

Mock authentication is accepted only when `NODE_ENV=development` and
`TELEMETRY_ADMIN_MOCK_AUTH=true` are both present.

Use the production D1 and R2 resources through Wrangler remote bindings while keeping authentication
local to the development server:

```powershell
$env:TELEMETRY_ADMIN_MOCK_AUTH='true'
$env:TELEMETRY_ADMIN_REMOTE_BINDINGS='true'
pnpm --filter @wisp/telemetry-dashboard dev
```

Remote mode is read-only at the application layer: the Admin API issues only `SELECT` statements
and a single registered-object R2 `GET`. It never exposes Cloudflare credentials to the browser.

For deterministic fixture scenarios, leave `TELEMETRY_ADMIN_REMOTE_BINDINGS` unset:

```powershell
$env:TELEMETRY_ADMIN_MOCK_AUTH='true'
$env:TELEMETRY_ADMIN_MOCK_SCENARIO='normal'
pnpm --filter @wisp/telemetry-dashboard dev
```

Supported deterministic scenarios are `normal`, `empty`, `api-error`, `no-sample`,
`budget-reached`, `unconfigured-auth`, and `forbidden`. All mock values are synthetic fixtures.

## Production deployment (Cloudflare Workers)

The dashboard runs on the `wisp-telemetry-dashboard` Worker with the custom domain
`admin.axlmc.org`. It uses the native D1 and R2 bindings declared in `wrangler.toml`; no data
credentials are sent to the browser.

Do not deploy the dashboard until every item below is complete:

1. Create a GitHub OAuth App controlled by the ` Wisp-Launcher` organization. Use
   `https://admin.axlmc.org` as its homepage and the callback URL shown by Cloudflare Zero Trust,
   normally `https://<team-name>.cloudflareaccess.com/cdn-cgi/access/callback`.
2. Add GitHub as a Cloudflare Zero Trust identity provider using that Client ID and Client Secret.
   Keep the Client Secret only in GitHub and Cloudflare.
3. Create a self-hosted Access application for `admin.axlmc.org` with an 8-hour session duration.
4. Create one Allow policy with `Include -> GitHub Organization -> Wisp-Launcher`. Do not add a
   bypass policy or a broader include rule.
5. Keep the Worker custom domain `admin.axlmc.org` bound to `wisp-telemetry-dashboard`. The DNS
   records for `axlmc.org` stay proxied through Cloudflare.

Build and deploy from `apps/telemetry-dashboard`:

```powershell
pnpm --filter @wisp/telemetry-dashboard build
pnpm --filter @wisp/telemetry-dashboard exec wrangler deploy
```

`nuxi build` uses Nitro's `cloudflare-module` preset by default and emits
`.output/server/index.mjs` plus `.output/public`, exactly what `wrangler.toml` references.

### Automatic deployment

`.github/workflows/deploy-telemetry-dashboard.yml` deploys on every push to `main` that touches
`apps/telemetry-dashboard/**`, and can also be triggered manually. It requires one repository
secret:

- `CLOUDFLARE_API_TOKEN`: an API token with Workers Scripts Edit and Workers Routes Edit
  permission for the account that owns `wisp-telemetry-dashboard`.

Do not add `[skip ci]` to commits that should reach production: GitHub Actions skips all workflows
for such commits, including this deployment.

### Cloudflare account usage check

The system page reports account-wide Workers, D1, and R2 usage through the GraphQL Analytics API.
Enable it once with two Worker secrets; secrets survive CI deploys because they live on the Worker,
not in the repository:

1. Create an API token at `dash.cloudflare.com` → My Profile → API Tokens with the
   `Account Analytics: Read` permission for the account that owns the telemetry resources.
2. From `apps/telemetry-dashboard`, run
   `pnpm exec wrangler secret put CLOUDFLARE_ANALYTICS_TOKEN` and paste that token.
3. Run `pnpm exec wrangler secret put CLOUDFLARE_ACCOUNT_ID` and paste the 32-character account ID.

The check stays fail-closed: without both values the row reports 尚未配置, and any dataset failure
only degrades the row. The token is used server-side for three read-only GraphQL queries per system
page load (Workers invocations 24h, D1 rows today, R2 operations 30d) and is never sent to the
browser.

### Unused Vercel target

An earlier Vercel project (`wisp-telemetry-dashboard`) exists, but its `admin.axlmc.org` domain
was never pointed at Vercel and traffic is served by the Cloudflare Worker. The Nitro `vercel`
preset and `server/utils/vercel-data-source.ts` remain as an offline fallback and are not part of
the production path. Do not enable both targets for the same hostname.

Verify an unauthenticated request is intercepted by Cloudflare Access, then verify
`/api/admin/session` reports `dataSource: production` after authentication.

The server fails closed when Access or the remote data source configuration is absent. A production
build ignores the mock provider even if a mock environment variable is present. All Admin API D1
operations are restricted to `SELECT`/`WITH` queries, and R2 reads require an object key previously
registered in D1.
