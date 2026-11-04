# TODO

Feature parity with a typical Khmer course + ebook store (reference: bakseyacademy.com — features and layout only; no copied branding, content or contact details). One task = one commit; run the checks in `CLAUDE.md` before each commit and tick the box in the same commit.

## A. Layout
- [x] A1 Footer: address, phones, email and social links (Facebook / Telegram / YouTube) from one config, `frontend/src/lib/site.ts` (placeholders until real details exist)
- [x] A2 Homepage slideshow: `banners` table, returned in `GET /api/home`; `Carousel.svelte` replaces the gradient hero

## B. Accounts (roadmap 2)
Login: phone (+855) + password, email optional. SvelteKit keeps the session token in an HttpOnly cookie and sends it to the API as `Authorization: Bearer`.
- [x] B1 Migration: `users.phone` (unique, E.164), `email` nullable, `sessions` table
- [x] B2 `POST /api/auth/register` (name, phone, password; argon2); `AppError::Validation`, `AppError::Conflict`
- [x] B3 `POST /api/auth/login`, `POST /api/auth/logout`, `GET /api/me` (Bearer token; `AppError::Unauthorized`)
- [x] B4 `/auth` page (login / register tabs, +855 prefix, logout action); `hooks.server.ts` resolves the `session` cookie via `/api/me` into `locals.user`
- [x] B5 Header account menu + logout; `/account` (profile, change password)
- [x] B6 Forgot password: Telegram support link (`site.supportUrl`)

## C. Commerce (roadmap 3)
Payment now: the user picks Bakong KHQR or ABA PayWay, scans that method's static QR image, and submits the transaction reference; an admin approves, which grants access. Orders: `pending_payment` → `awaiting_approval` → `paid` | `rejected` (resubmittable); `cancelled` from `pending_payment`; a free order is `paid` at checkout.
- [x] C1 Migration: `payment_methods` (QR image, account), `cart_items`, `orders`, `order_items` (price snapshot), `enrollments`, `ebook_purchases`
- [x] C2 Cart API (`GET/POST/DELETE /api/cart`), add-to-cart on detail pages, header cart icon with count
- [x] C3 `/cart` + payment-method picker → `POST /api/orders`; `/orders/{id}` shows QR + amount, reference form, cancel, status banners
- [x] C4 Admin approval: `AdminUser` extractor (`AppError::Forbidden`), `/api/admin/orders` list / approve / reject, `/admin/orders` page
- [x] C5 `/account/orders`, `/account/library`, `owned` flag on detail pages ("Continue learning" / "Read")

## D. Learning (roadmap 4)
- [x] D1 `GET /api/courses/{slug}/lessons/{id}`: signed video URL for enrolled users only
- [x] D2 Lesson player page + curriculum sidebar
- [x] D3 Progress tracking (`lesson_progress`, mark complete, % on cards)
- [x] D4 Ebook signed download / reader for owners

## E. Discovery & admin (roadmap 5)
- [x] E1 Reviews write API (owners only) + form on detail pages
- [x] E2 `/for-you` recommendations (same category + co-purchase, server-side)
- [x] E3 Admin CRUD: courses, sections, lessons, ebooks, banners, categories, publish toggle
- [x] E4 Search upgrade (Postgres trigram first, Meilisearch if needed) — trigram is enough for now; `/search` covers courses + ebooks

## F. i18n (roadmap 6)
- [x] F1 Paraglide km/en + header language switcher (storefront, account, learning; admin stays Khmer)

## G. Telegram channel storage
Lesson videos and ebook files live as posts in a **private Telegram channel**; a bot that is admin of the channel stores and fetches them. It sits next to the current providers (Bunny Stream for video, S3/R2 for files) as one more `provider`, so existing content keeps working.

Constraints that shape the design:
- The public Bot API only downloads files ≤ 20 MB and uploads ≤ 50 MB. Course videos are bigger, so run a **self-hosted Telegram Bot API server** (`telegram-bot-api`, `--local`: up to 2 GB, files read from its disk) and point `TELEGRAM_API_URL` at it.
- Telegram file URLs contain the bot token and never expire. They must never reach the browser: every byte is **streamed through our server** after the access check (keeps the CLAUDE.md invariant: paid content stays server-side until enrollment).
- Telegram is not a CDN: expect rate limits and slower first bytes than Bunny/R2. Keep the provider switchable per item, and check Telegram's terms for this use before launch.

- [x] G1 Config + client: `TELEGRAM_BOT_TOKEN`, `TELEGRAM_CHANNEL_ID`, `TELEGRAM_API_URL` (default public API) in `config.rs`; `src/telegram.rs` with `send_document` / `send_video` (multipart upload to the channel), `get_file` (file_id → file_path) and a byte stream for a file_path with HTTP `Range` support; startup warning when unset, like `BUNNY_*` / `STORAGE_*`
- [x] G2 Migration `media_files`: `provider` (`bunny` | `s3` | `telegram`), `telegram_file_id`, `telegram_file_unique_id`, `telegram_message_id`, `mime_type`, `size_bytes`, `file_name`, `duration_seconds`; `lessons.media_id` / `ebooks.media_id` (nullable; existing `video_id` / `file_key` stay for bunny / s3)
- [x] G3 Admin upload: `POST /api/admin/media` (multipart, admin) → uploads to the channel → `media_files` row; lesson and ebook admin forms get "upload file" next to the Video ID / file key fields. Uploads stream to Telegram rather than buffering whole files in memory
- [x] G4 Import existing channel posts: bot receives `channel_post` updates (webhook `POST /api/telegram/webhook`, verified with `X-Telegram-Bot-Api-Secret-Token`) and registers each video/document as a `media_files` row; admin picks it from a "media library" list (`GET /api/admin/media?q=`) when editing a lesson / ebook
- [x] G5 Streaming endpoint: lesson / ebook-file endpoints (D1, D4) return a short-lived HMAC-signed media URL for `telegram` items (`MEDIA_SIGNING_KEY`, same TTLs as today) instead of a Bunny / S3 URL; `GET /api/media/{token}` verifies signature + expiry, resolves `file_id` → `file_path` (cache the path; Telegram keeps it valid ≥ 1 h), and streams with `Range` / `206 Partial Content`, `Content-Type`, `Content-Disposition` (inline / attachment)
- [x] G6 SvelteKit proxy: `/media/[token]/+server.ts` forwards the request (incl. `Range`) to `/api/media/{token}` and pipes the response body, so the browser still never talks to the Rust API; document this as the one streaming exception next to the `api()` rule in `CLAUDE.md`
- [x] G7 Player + reader: lessons whose media is `telegram` play in a native `<video>` (seeking via Range) instead of the Bunny iframe; ebook reader / download use the same proxy URL. Mark complete keeps working unchanged
- [x] G8 Ops: README section (create channel, add bot as admin, run `telegram-bot-api --local` via `docker-compose.yml`, env vars), on-disk cache limits for the local Bot API server, and a smoke test that uploads → plays with seeking → downloads a > 20 MB file

## H. Launch readiness
- [x] H1 CI: GitHub Actions workflow running `cargo check`, `cargo clippy -- -D warnings`, `cargo test` (backend) and `bun run check` (frontend) on every push / PR
- [ ] H2 Access-control integration tests (`sqlx::test` on a throwaway DB): lesson `video_id`, ebook `file_key` and Telegram ids never reach non-owners; owners keep access after unpublish; approving an order grants enrollment / purchase exactly once
- [ ] H3 Buyer notification on order approve / reject (Telegram bot message and/or header notice), so buyers don't have to poll `/orders/{id}`
- [ ] H4 SEO: `og:title` / `og:description` / `og:image` on course and ebook detail pages (for Facebook / Telegram link previews), `sitemap.xml` of published items, `robots.txt`
- [ ] H5 Admin dashboard at `/admin`: count of orders awaiting approval, revenue over time, top-selling courses / ebooks

## I. Growth
- [ ] I1 Coupons / discount codes: `coupons` table (percent or fixed cents, expiry, usage limit), applied at checkout, discount snapshot on the order
- [ ] I2 Completion certificates: PDF (km / en) issued when course progress reaches 100%, listed in `/account/library`
- [ ] I3 Wishlist for logged-in users (heart on cards and detail pages, `/account/wishlist`)
- [ ] I4 Lesson Q&A / comments: enrolled users ask, instructors / admins answer
- [ ] I5 Admin user management: search users, grant / revoke enrollments and ebook purchases manually (refunds, support), promote to admin

## J. Polish
- [ ] J1 Resume a course at the last lesson watched ("Continue learning" goes to it)
- [ ] J2 Bundles: several courses / ebooks sold at one price
- [ ] J3 Cleanup job: delete expired sessions and cancel long-abandoned `pending_payment` orders

## Later
- [ ] Automatic payment confirmation: Bakong KHQR dynamic QR + status check, ABA PayWay checkout + pushback webhook (same grant path as admin approval)
- [x] Payment screenshot upload: the user attaches a screenshot on `/orders/{id}`; the bot posts it (with order id, buyer, amount, reference) to a private **admin** Telegram channel/group, which also notifies admins; `orders.payment_proof_media_id` → shown in `/admin/orders` through the G5/G6 proxy (builds on G1–G2)
- [ ] Self-service password reset (SMS OTP)
- [ ] Google sign-in (server-side OAuth; `users.google_sub`)
