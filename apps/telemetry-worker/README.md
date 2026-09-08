# Wisp telemetry worker

This Worker accepts only opted-in launcher heartbeat and error batches. It stores anonymous metadata in D1 and bounded, sanitized error context in R2.

## Provisioning

```sh
pnpm exec wrangler d1 create wisp-telemetry
pnpm exec wrangler r2 bucket create wisp-telemetry-errors
pnpm exec wrangler secret put INSTALLATION_HMAC_SECRET
pnpm exec wrangler d1 migrations apply wisp-telemetry --remote
pnpm exec wrangler r2 bucket lifecycle add wisp-telemetry-errors telemetry-errors-30d errors/ --expire-days 30
```

Replace `database_id` in `wrangler.toml` and keep the Worker on the Free plan. The production custom domain is declared in `wrangler.toml`. Configure Cloudflare usage notifications at 50%, 75%, and 90% for Workers, D1, and R2. Do not enable Workers Paid.

`STORE_ERROR_CONTEXT=false` stops all R2 writes while heartbeat and error aggregates continue. The code clamps R2 creation to 2,000 objects per UTC day, three samples per fingerprint/version/day, and 16 KiB uncompressed per object even if environment variables request larger limits. Error detail rows in D1 are additionally sampled to two per fingerprint/version/day; occurrence counts stay exact.

## Storage design

D1 stays within the Free plan's daily limits (5M rows read / 100k rows written) by avoiding per-event aggregation triggers:

- The worker upserts per-group counters into `error_daily` (exact occurrence counts, distinct installs via `error_daily_installations`) and writes at most two sampled `error_reports` detail rows per group per day.
- `daily_active` inserts feed `wau_seen` / `mau_seen` / `daily_active_dims` through a single lightweight trigger for O(1) WAU/MAU and distribution reads.
- A nightly cron (`runMaintenance`) rebuilds `daily_totals`, `error_groups`, and the 7/30/90-day `error_range_stats` for the previous day, rolls the active windows, and applies retention. Dashboard reads are cached in-process.

Schema changes are forward-only migrations in `migrations/`; never edit an applied migration.
