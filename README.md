# Reanit LMS

Online course + ebook store (Khmer-first).

- `backend/` — Rust API: **Axum 0.8 + SQLx 0.8 + PostgreSQL**
- `frontend/` — **SvelteKit (Svelte 5) + Tailwind v4**, server-rendered, talks to the API server-side

## Prerequisites

- Rust (stable), Bun (or Node + pnpm), PostgreSQL 16+ (Homebrew `postgresql@18` works; `docker-compose.yml` is optional)

## Run

```sh
# 1. database (Homebrew Postgres)
createdb reanit_lms            # skip if it already exists

# 2. API — applies migrations on start
cd backend
cp .env.example .env           # adjust DATABASE_URL; BUNNY_* lesson videos, STORAGE_* ebook files, TELEGRAM_* channel storage
cargo run                      # http://127.0.0.1:8080/api/health

# 3. demo data (dev only — truncates tables!)
psql postgres://localhost/reanit_lms -f seeds/dev.sql

# 4. web
cd ../frontend
cp .env.example .env
bun install
bun run dev                    # http://localhost:5173
```

The API runs a cleanup pass on start and then hourly: it deletes expired sessions and cancels `pending_payment` orders older than `ORDER_ABANDON_DAYS` (optional env var, 1–365, default 7), so their items can be ordered again and coupon uses are released. Orders awaiting approval are never touched.

Tests: `cd backend && cargo test` runs unit tests and the integration tests in `src/tests/`, which create a throwaway database per test on the server in `DATABASE_URL` (the role needs `CREATEDB`). CI (`.github/workflows/ci.yml`) runs them against a Postgres service, plus clippy, `bun run build` and `bun run check`.

## Telegram channel storage

Lesson videos and ebook files can live as posts in a private Telegram channel instead of Bunny / R2. The bot stores and fetches them; the API streams every byte through signed, expiring `/media/{token}` links, so the bot token and Telegram file URLs never reach the browser.

1. **Bot + channel.** Create a bot with [@BotFather](https://t.me/BotFather), create a **private** channel, and add the bot as an admin that can post. The channel id is the `-100…` number (e.g. forward a channel post to [@userinfobot](https://t.me/userinfobot)).
2. **API env** (`backend/.env`): `TELEGRAM_BOT_TOKEN`, `TELEGRAM_CHANNEL_ID`, `MEDIA_SIGNING_KEY` (required: any long random string, the same on every instance), `TELEGRAM_WEBHOOK_SECRET`, and `TELEGRAM_ADMIN_CHAT_ID`: the admins' group/channel (bot added) that receives buyers' payment screenshots with the order details, so admins get notified (defaults to the storage channel).
3. **Large files: self-hosted Bot API.** The public API only lets bots download ≤ 20 MB (so uploads are capped at 20 MB in that mode), too small for course videos. Run the `--local` server (uploads ≤ 2000 MB) with `api_id` / `api_hash` from <https://my.telegram.org>:
   ```sh
   TELEGRAM_API_ID=… TELEGRAM_API_HASH=… docker compose --profile telegram up -d telegram-bot-api
   ```
   then point the API at it, and at the shared data directory it reads files from (the 2000 MB upload cap only applies once `TELEGRAM_LOCAL_DIR` is set; without it the API assumes the public API's 20 MB):
   ```sh
   TELEGRAM_API_URL=http://127.0.0.1:8081
   TELEGRAM_LOCAL_DIR=/absolute/path/to/repo/.telegram-bot-api   # mounted in the container as /var/lib/telegram-bot-api
   ```
   A bot can only use one server at a time: call `logOut` on the public API once before switching (`curl https://api.telegram.org/bot$TOKEN/logOut`).
4. **Import channel posts (optional).** Point the bot's webhook at the site; videos/documents posted in the channel then show up in **Admin → ឯកសារ**:
   ```sh
   curl "$TELEGRAM_API_URL/bot$TOKEN/setWebhook" -d url=https://<site>/telegram/webhook \
        -d secret_token=$TELEGRAM_WEBHOOK_SECRET -d 'allowed_updates=["channel_post"]'
   ```
5. **Upload size on the web side.** Uploads stream through SvelteKit (`/admin/media/upload`; payment screenshots ≤ 10 MB go through the order page's form action). adapter-node rejects bodies over `BODY_SIZE_LIMIT` (default 512 KB): start the built frontend with e.g. `BODY_SIZE_LIMIT=2200M node build`, and raise any reverse-proxy limit too (nginx `client_max_body_size 2200m`). `hooks.server.ts` still caps every other route at 12 MB, so only the admin upload route accepts large bodies.
6. **Disk.** The local server keeps a copy of every file it serves under `.telegram-bot-api/<bot>/`. The `telegram-cache-prune` service (same compose profile) deletes copies older than `TELEGRAM_CACHE_DAYS` (default 7) every hour; a pruned file is fetched from Telegram again on its next request.
7. **Check it:** `SITE=http://localhost:5173 API=http://127.0.0.1:8080 SESSION=<admin session cookie> scripts/telegram-smoke.sh` uploads a 25 MB file, streams it back and seeks (`SIZE_MB=` to change).

Admins upload from the lesson / ebook editors ("ផ្ទុកឡើង Telegram") or pick from the library; uploaded media wins over a Bunny video id / S3 file key. Telegram is not a CDN (rate limits, slower first byte), so keep Bunny / R2 for heavy traffic and check Telegram's terms for this use before launch.

## Automatic payments (Bakong KHQR, ABA PayWay)

Without credentials, both methods work as before: the buyer scans the static QR from `payment_methods`, sends a reference or screenshot, and an admin approves. Setting a provider's env vars makes its method (`bakong_khqr` / `aba_payway`) confirm itself:

- **Bakong KHQR** — `BAKONG_TOKEN` (Bakong Open API token; it expires every 90 days, renew it), `BAKONG_ACCOUNT_ID` (`name@bank`), `BAKONG_MERCHANT_NAME` (≤ 25 ASCII chars), optional `BAKONG_MERCHANT_CITY` (default `Phnom Penh`), `BAKONG_MERCHANT_ID` + `BAKONG_ACQUIRING_BANK` (merchant QR instead of individual), `BAKONG_API_URL` (default `https://api-bakong.nbc.gov.kh`). The order page shows a dynamic KHQR for the exact total (valid 15 min) and reloads every 5 s; the API looks the payment up by the QR's MD5.
- **ABA PayWay** — `PAYWAY_MERCHANT_ID`, `PAYWAY_API_KEY`, `PAYWAY_API_URL` (`https://checkout-sandbox.payway.com.kh` while testing), `SITE_URL` (public `https://` storefront origin). The order page posts a signed (HMAC-SHA512) checkout to PayWay; PayWay's pushback arrives at `/payments/payway/pushback` (relayed to the API) and buyers return to `/orders/{id}`.

Security model: the API never trusts the browser or a pushback. An order is paid only when the provider's own lookup (Bakong `check_transaction_by_md5`, PayWay `check-transaction-2`) reports the attempt paid **and** the amount, currency and (Bakong) receiving account match; a mismatch is logged and the attempt closed. Confirmation goes through the same transaction as admin approval (order row locked, items granted once, buyer notified), so concurrent checks, repeated pushbacks and an admin approving at the same time settle the order once. Lookups happen when the buyer's page reloads, on pushback, before a cancel (a payment made just before cancelling wins), and every 20 s in the background for 2 days. A payment that arrives for an order that was already cancelled is logged as an error for a manual refund. Verify both integrations in the providers' sandboxes before going live: the response shapes follow their docs, and Bakong's API may only answer requests from Cambodian IPs.

## i18n

Khmer by default; the header button switches to English (cookie `PARAGLIDE_LOCALE`). Copy lives in `frontend/messages/km.json` / `en.json`; Paraglide compiles it into `src/lib/paraglide/` on `bun run dev` / `build` (it fetches its inlang plugins from jsDelivr on first compile). The admin area is Khmer-only.

## Admin

Make an account an admin (it then sees **ផ្ទាំងគ្រប់គ្រង** in the account menu → `/admin`: dashboard (approval queue, revenue, best sellers), orders, lesson questions, bundles, users (roles, manual grants / revocations), coupons, courses + curriculum, ebooks, banners, categories, instructors):

```sh
psql postgres://localhost/reanit_lms -c "UPDATE users SET role = 'admin' WHERE phone = '+85512345678'"
```

Ebook files live in the S3-compatible bucket from `STORAGE_*` (e.g. Cloudflare R2); set `ebooks.file_key` to the object key. Lesson `video_id` is the Bunny Stream video GUID.

Payment QR images and account details live in `payment_methods` (`qr_image_url` can be a path under `frontend/static/` or any URL); the dev seed points at placeholders.

## API

Authenticated routes take `Authorization: Bearer <token>` (the token from register/login).

| Method | Path | Notes |
|---|---|---|
| GET | `/api/health` | DB ping |
| GET | `/api/home` | published banners + 8 newest courses + ebooks |
| GET | `/api/categories` | |
| GET | `/api/certificates/{id}` | public → `{id, full_name, course_title, instructor_name, issued_at, course_slug}` (verification link; names are snapshots at issue time). Issued automatically when an enrolled learner completes every lesson (`PUT …/complete`); listed as `certificate_id` in `/api/me/library` |
| GET | `/api/sitemap` | `{courses, ebooks, bundles}`: `{slug, updated_at}` of every published item (feeds the storefront's `/sitemap.xml`) |
| GET | `/api/courses?q=&category=&page=&per_page=` | paged; `q` matches title/instructor by substring or typo-tolerant trigram similarity (`pg_trgm`), best match first |
| GET | `/api/courses/{slug}` | detail + curriculum (video id only for preview lessons); `reviews` (latest 10); optional Bearer → `owned`, `last_lesson_id` (opened last; recorded by the lesson endpoint for enrolled users), `my_review`, `completed_lesson_ids` (enrolled users also see unpublished courses) |
| PUT / DELETE | `/api/courses/{slug}/review` | Bearer, owners only; PUT `{rating: 1–5, comment?}` → review (upsert), DELETE → 204; 403 not enrolled, 422 `invalid_rating` / `invalid_comment` |
| GET | `/api/courses/{slug}/lessons/{id}` | optional Bearer → lesson + signed `video_url` (`expires_at`) + `video_type`: `embed` (Bunny iframe) or `file` (`/media/{token}` for uploaded Telegram media, which wins), `completed`; enrolled users (even if unpublished) or preview lessons; 401 signed out, 403 not enrolled |
| PUT / DELETE | `/api/courses/{slug}/lessons/{id}/complete` | Bearer → 204; mark / unmark complete (PUT needs enrollment, 403 otherwise) |
| GET / POST | `/api/courses/{slug}/lessons/{id}/questions` | Q&A on a lesson → `{questions, can_answer}` (latest 100, newest first, each with `answers` oldest first; `mine` flags the viewer's own). GET: same access as the lesson (staff always); POST `{body}` (≤ 2000 chars): enrolled learners or staff → 201; 403 otherwise, 422 `invalid_body`. Staff = admins and the user linked to the course's instructor (`instructors.user_id`) |
| POST | `/api/questions/{id}/answers` | Bearer, staff of the course, `{body}` (≤ 4000) → 201 lesson Q&A; 403 otherwise |
| DELETE | `/api/questions/{id}`, `/api/answers/{id}` | Bearer, author or staff → 204; 403 otherwise |
| GET | `/api/bundles?page=&per_page=` | paged published, non-empty bundles, newest first: `{id, slug, title, summary, cover_url, price_cents, sale_price_cents, item_count, value_cents}` (`value_cents` = items bought one by one) |
| GET | `/api/bundles/{slug}` | bundle + `description`, `courses` / `ebooks` (card shape); optional Bearer → `owned` (has everything in it; owners also see it after unpublishing) |
| GET | `/api/ebooks?q=&category=&page=&per_page=` | paged; `q` matches title/author like courses |
| GET | `/api/ebooks/{slug}` | detail (file key never exposed), `reviews`; optional Bearer → `owned`, `my_review` (owners also see unpublished ebooks) |
| PUT / DELETE | `/api/ebooks/{slug}/review` | same as course reviews, for buyers |
| GET | `/api/ebooks/{slug}/file?disposition=inline\|attachment` | Bearer, owners only → `{url, expires_at}`: `/media/{token}` for uploaded Telegram media, else a presigned (1 h) S3/R2 link; 403 not bought, 404 no file, 503 `storage_unavailable` |
| POST | `/api/auth/register` | `{full_name, phone, password}` → 201 `{token, max_age, user}`; 409 `phone_taken`, 422 `invalid_phone` / `invalid_name` / `weak_password` |
| POST | `/api/auth/login` | `{phone, password}` → `{token, max_age, user}`; 401 `invalid_credentials`; 429 `too_many_attempts` after 10 wrong passwords for a number within 15 min |
| POST | `/api/auth/logout` | Bearer → 204, deletes the session |
| GET | `/api/me` | Bearer → user; 401 `unauthorized` |
| PATCH | `/api/me` | Bearer, `{full_name, email}` (empty email clears it) → user; 409 `email_taken`, 422 `invalid_email` / `invalid_name` |
| POST | `/api/me/password` | Bearer, `{current_password, new_password}` → 204, signs out other sessions; 422 `wrong_password` / `weak_password` |
| GET | `/api/me/orders` | Bearer → own orders with items, newest first |
| GET | `/api/me/library` | Bearer → `{courses, ebooks}` the user owns (card shape; courses add `lesson_count`, `completed_lessons`, `certificate_id`) |
| GET | `/api/me/notifications` | Bearer → `{items, unread_count}`: latest 20 notices, newest first (`kind`: `order_paid`, or `order_rejected` with the admin's `note`; `order_id`, `read_at`) |
| POST | `/api/me/notifications/read` | Bearer → 204, marks all read |
| GET | `/api/me/wishlist` | Bearer → `{courses, ebooks}` saved items that are still published (card shape), most recently saved first |
| GET | `/api/me/wishlist/ids` | Bearer → `{course_ids, ebook_ids}` (fills in the hearts) |
| PUT / DELETE | `/api/me/wishlist/{courses\|ebooks}/{id}` | Bearer → 204; save (published items only, else 404; idempotent) / unsave |
| GET | `/api/recommendations` | optional Bearer → `{courses, ebooks, personalized}`: 8 unowned published items each, ranked by co-purchase, then shared category, then popularity (signed out: popularity) |
| GET | `/api/cart?coupon=` | Bearer → `{items, subtotal_cents, discount_cents, total_cents, coupon}` (published items; prices use the sale price); `coupon` previews a discount code (422 `invalid_coupon` / `coupon_expired` / `coupon_used_up`) |
| POST | `/api/cart` | Bearer, `{course_id}`, `{ebook_id}` or `{bundle_id}` → cart (a bundle is one line; paying for it grants everything inside; owning all of it counts as `already_owned`); 404, 409 `already_owned` / `already_ordered` (in an unpaid order), 422 `invalid_item` |
| DELETE | `/api/cart/{id}` | Bearer → cart |
| GET | `/api/payment-methods` | active methods with QR image and account; `automatic` when the method's provider is configured |
| POST | `/api/orders` | Bearer, `{payment_method, coupon_code?}` → 201 order (cart → `pending_payment`; free → `paid` + granted); 409 `already_ordered`, 422 `cart_empty` / `invalid_payment_method` / `invalid_coupon` / `coupon_expired` / `coupon_used_up`; orders carry `discount_cents` and `coupon_code` (total is after the discount; a coupon that covers everything makes the order free) |
| GET | `/api/orders/{id}` | Bearer, own orders only → order with items + payment method |
| POST | `/api/orders/{id}/submit` | Bearer, `{payment_ref}` → `awaiting_approval` (from `pending_payment` or `rejected`); 409 `invalid_status`, 422 `invalid_payment_ref` |
| POST | `/api/orders/{id}/proof` | Bearer, own order in `pending_payment` / `awaiting_approval` / `rejected`; multipart `size` then `file` (PNG/JPEG/WebP ≤ 10 MB) → `awaiting_approval` order with `payment_proof_url` (a screenshot counts as submitting, like a reference); the screenshot is posted with the order details to `TELEGRAM_ADMIN_CHAT_ID`; 409 `invalid_status`, 422 `invalid_image` / `invalid_size` / `file_too_large` / `invalid_form`, 503 `telegram_unavailable` / `telegram_failed` |
| POST | `/api/orders/{id}/khqr` | Bearer, own payable order paying with `bakong_khqr` → `{qr, amount_cents, expires_at}`: the current dynamic KHQR (reused while valid > 1 min, else a new one, 15 min); 409 `invalid_status`, 422 `invalid_payment_method`, 503 `payment_unavailable` (Bakong not configured) |
| POST | `/api/orders/{id}/payway` | Bearer, own payable order paying with `aba_payway` → `{action_url, fields}` for the browser to POST (multipart) to PayWay; a new `tran_id` per attempt; errors as `khqr` |
| POST | `/api/orders/{id}/check-payment` | Bearer, own order → order, after asking the provider about its open attempts (paid → `paid`, `paid_via` `bakong` / `payway`, `payment_ref` = provider transaction) |
| POST | `/api/payments/payway/pushback` | PayWay only (via the storefront relay), JSON or form body with `tran_id` → re-checks that attempt with PayWay; 404 for unknown attempts |
| POST | `/api/orders/{id}/cancel` | Bearer → `cancelled` (from `pending_payment` only) |
| GET | `/api/admin/stats?days=` | admin → dashboard: `awaiting_approval`, `pending_payment`, `revenue_cents_all_time`, `users`, `new_users`, and for the last `days` (7–365, default 30; Phnom Penh days) `revenue_cents`, `paid_orders`, zero-filled `daily` `{day, revenue_cents, orders}`, `top_courses` / `top_ebooks` (5 each: `{id, slug, title, sales, revenue_cents}`); 422 `invalid_days` |
| GET | `/api/admin/questions?unanswered=&page=&per_page=` | admin → paged questions across courses with course / lesson titles and `answer_count`; `unanswered` (default true) lists unanswered ones oldest first |
| GET | `/api/admin/users?q=&role=&page=&per_page=` | admin → paged users, newest first (`q`: name / phone / email / id; `role`: `student` \| `instructor` \| `admin`) with `course_count`, `ebook_count` |
| GET | `/api/admin/users/{id}` | admin → user + owned `courses` / `ebooks` (`{id, slug, title, granted_at, order_id}`; `order_id` null = manual grant) + `grantable_courses` / `grantable_ebooks` |
| PUT | `/api/admin/users/{id}/role` | admin, `{role}` → 204; 409 `own_role` (admins can't change their own), 422 `invalid_role` |
| PUT / DELETE | `/api/admin/users/{id}/{courses\|ebooks}/{item}` | admin → 204; grant without an order (idempotent, also unpublished items; removes it from the cart) / revoke (orders, progress and reviews are kept) |
| GET | `/api/admin/orders?status=&q=&page=&per_page=` | admin → paged orders with buyer + items; `awaiting_approval` oldest first; 403 `forbidden` |
| POST | `/api/admin/orders/{id}/approve` | admin → `paid` + grants enrollments / ebooks and notifies the buyer (idempotent; also from `pending_payment` / `rejected`) |
| POST | `/api/admin/orders/{id}/reject` | admin, `{note}` → `rejected` (from `awaiting_approval`); the buyer is notified, sees the note and can resubmit; 409 `invalid_status`, 422 `invalid_note` |
| GET / POST | `/api/admin/courses?q=&page=` | admin → paged courses incl. drafts / create `{slug, title, summary?, description, cover_url?, instructor_id, category_id?, price_cents, sale_price_cents?, published}` → 201; 409 `slug_taken`, 422 `invalid_slug` / `invalid_price` / `invalid_url` / `invalid_reference` |
| GET / PUT / DELETE | `/api/admin/courses/{id}` | admin → course + curriculum (lessons include `video_id`); DELETE 409 `has_buyers` once enrolled/ordered |
| POST | `/api/admin/courses/{id}/publish` | admin, `{published}` → 204 |
| POST | `/api/admin/courses/{id}/sections` | admin, `{title}` → course (appended) |
| PUT / DELETE | `/api/admin/sections/{id}` | admin, `{title}` → course |
| POST | `/api/admin/sections/{id}/lessons` | admin, `{title, video_id?, duration_seconds, is_preview}` → course (appended) |
| PUT / DELETE | `/api/admin/lessons/{id}` | admin → course |
| POST | `/api/admin/sections/{id}/move`, `/api/admin/lessons/{id}/move` | admin, `{direction: up\|down}` → course |
| GET / POST | `/api/admin/ebooks?q=&page=` | admin → paged ebooks incl. drafts (`has_file`) / create (same fields as courses minus instructor, plus `author_name`, `page_count?`, `file_key?`) |
| GET / PUT / DELETE | `/api/admin/ebooks/{id}` | admin; `file_key` is write-only (blank keeps, `remove_file: true` clears); DELETE 409 `has_buyers` |
| POST | `/api/admin/ebooks/{id}/publish` | admin, `{published}` → 204 |
| GET | `/api/media/{token}` | streams a Telegram-stored file; `token` (from the lesson / ebook-file endpoints) is HMAC-signed with `MEDIA_SIGNING_KEY` and expires (4 h video, 1 h files); honors `Range` → 206 / 416; 404 for bad or expired tokens. Only video/audio/PDF/raster images render inline (anything else downloads); responses carry `Content-Security-Policy: sandbox` |
| GET | `/api/admin/media?q=&kind=video\|document&page=&per_page=` | admin → paged media library of course content (newest first, ≤ 200 per page; payment screenshots never listed) |
| POST | `/api/telegram/webhook` | Telegram only (`X-Telegram-Bot-Api-Secret-Token` = `TELEGRAM_WEBHOOK_SECRET`, else 401; always 401 while no secret is set; 404 without Telegram): videos/documents posted in the storage channel are imported as media (idempotent) |
| POST | `/api/admin/media` | admin, multipart `size` then `file` → 201 media; the file is streamed to the Telegram storage channel (≤ 20 MB public Bot API, ≤ 2000 MB with a `--local` server); 422 `invalid_size` / `file_too_large` / `missing_file`, 503 `telegram_unavailable` / `telegram_failed`. Lessons / ebooks take `media_id`, which wins over `video_id` / `file_key` |
| GET / POST | `/api/admin/bundles?q=&page=` | admin → paged bundles incl. drafts (`item_count`, paid `sales`) / create `{slug, title, summary?, description, cover_url?, price_cents, sale_price_cents?, published}` → 201; 409 `slug_taken` |
| GET / PUT / DELETE | `/api/admin/bundles/{id}` | admin → bundle + `courses` / `ebooks` + `other_courses` / `other_ebooks` (for the add pickers); DELETE 409 `has_buyers` once ordered |
| PUT / DELETE | `/api/admin/bundles/{id}/{courses\|ebooks}/{item}` | admin → bundle; add (appended, idempotent) / remove an item. New buyers get the current contents; past buyers keep their grants |
| GET / POST, PUT / DELETE `/{id}` | `/api/admin/coupons` | admin, `{code, percent_off? \| amount_off_cents?, expires_at?, max_uses?, active}` (code upper-cased, 3–32 of `A–Z 0–9 _ -`; exactly one discount) → coupon with `uses` (non-cancelled orders) and `discount_given_cents`; 409 `code_taken`, 422 `invalid_coupon` / `invalid_discount` / `invalid_max_uses`. Deleting keeps orders' discount snapshot |
| GET / POST, PUT / DELETE `/{id}` | `/api/admin/categories` | admin, `{slug, name_km, name_en, position}`; 409 `slug_taken` |
| GET / POST, PUT / DELETE `/{id}` | `/api/admin/instructors` | admin, `{name, bio?, avatar_url?}`; DELETE 409 `in_use` while courses reference it |
| GET / POST, PUT / DELETE `/{id}` | `/api/admin/banners` | admin, `{title, subtitle?, image_url?, link_url?, position, published}`; URLs must be `/path` or `http(s)://` |

## Roadmap

Task-level breakdown: [`TODO.md`](TODO.md).

1. ~~Catalog: schema, courses, ebooks, categories, ratings~~
2. ~~Auth (phone + argon2, session cookies), account page~~
3. ~~Cart → orders → QR payment (Bakong KHQR / ABA PayWay) + admin approval → enrollments~~ (automatic confirmation: see TODO "Later")
4. ~~Lesson player (Bunny Stream signed URLs), progress tracking, ebook signed downloads~~
5. ~~Reviews write API, search upgrade (Postgres trigram), "for you" recommendations, admin~~
6. ~~i18n (Paraglide: km/en)~~
