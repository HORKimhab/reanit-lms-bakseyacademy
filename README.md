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

## Telegram channel storage

Lesson videos and ebook files can live as posts in a private Telegram channel instead of Bunny / R2. The bot stores and fetches them; the API streams every byte through signed, expiring `/media/{token}` links, so the bot token and Telegram file URLs never reach the browser.

1. **Bot + channel.** Create a bot with [@BotFather](https://t.me/BotFather), create a **private** channel, and add the bot as an admin that can post. The channel id is the `-100…` number (e.g. forward a channel post to [@userinfobot](https://t.me/userinfobot)).
2. **API env** (`backend/.env`): `TELEGRAM_BOT_TOKEN`, `TELEGRAM_CHANNEL_ID`, `MEDIA_SIGNING_KEY` (required: any long random string, the same on every instance), `TELEGRAM_WEBHOOK_SECRET`, and `TELEGRAM_ADMIN_CHAT_ID`: the admins' group/channel (bot added) that receives buyers' payment screenshots with the order details, so admins get notified (defaults to the storage channel).
3. **Large files: self-hosted Bot API.** The public API only lets bots download ≤ 20 MB (so uploads are capped at 20 MB in that mode), too small for course videos. Run the `--local` server (uploads ≤ 2000 MB) with `api_id` / `api_hash` from <https://my.telegram.org>:
   ```sh
   TELEGRAM_API_ID=… TELEGRAM_API_HASH=… docker compose --profile telegram up -d telegram-bot-api
   ```
   then point the API at it, and at the shared data directory it reads files from:
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
6. **Disk.** The local server keeps a copy of every file it serves under `.telegram-bot-api/<bot>/`. Prune old copies, e.g. daily: `find .telegram-bot-api -type f -mtime +7 -delete`; a pruned file is fetched from Telegram again on the next request.
7. **Check it:** `SITE=http://localhost:5173 API=http://127.0.0.1:8080 SESSION=<admin session cookie> scripts/telegram-smoke.sh` uploads a 25 MB file, streams it back and seeks (`SIZE_MB=` to change).

Admins upload from the lesson / ebook editors ("ផ្ទុកឡើង Telegram") or pick from the library; uploaded media wins over a Bunny video id / S3 file key. Telegram is not a CDN (rate limits, slower first byte), so keep Bunny / R2 for heavy traffic and check Telegram's terms for this use before launch.

## i18n

Khmer by default; the header button switches to English (cookie `PARAGLIDE_LOCALE`). Copy lives in `frontend/messages/km.json` / `en.json`; Paraglide compiles it into `src/lib/paraglide/` on `bun run dev` / `build` (it fetches its inlang plugins from jsDelivr on first compile). The admin area is Khmer-only.

## Admin

Make an account an admin (it then sees **ផ្ទាំងគ្រប់គ្រង** in the account menu → `/admin`: orders, courses + curriculum, ebooks, banners, categories, instructors):

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
| GET | `/api/courses?q=&category=&page=&per_page=` | paged; `q` matches title/instructor by substring or typo-tolerant trigram similarity (`pg_trgm`), best match first |
| GET | `/api/courses/{slug}` | detail + curriculum (video id only for preview lessons); `reviews` (latest 10); optional Bearer → `owned`, `my_review`, `completed_lesson_ids` (enrolled users also see unpublished courses) |
| PUT / DELETE | `/api/courses/{slug}/review` | Bearer, owners only; PUT `{rating: 1–5, comment?}` → review (upsert), DELETE → 204; 403 not enrolled, 422 `invalid_rating` / `invalid_comment` |
| GET | `/api/courses/{slug}/lessons/{id}` | optional Bearer → lesson + signed `video_url` (`expires_at`) + `video_type`: `embed` (Bunny iframe) or `file` (`/media/{token}` for uploaded Telegram media, which wins), `completed`; enrolled users (even if unpublished) or preview lessons; 401 signed out, 403 not enrolled |
| PUT / DELETE | `/api/courses/{slug}/lessons/{id}/complete` | Bearer → 204; mark / unmark complete (PUT needs enrollment, 403 otherwise) |
| GET | `/api/ebooks?q=&category=&page=&per_page=` | paged; `q` matches title/author like courses |
| GET | `/api/ebooks/{slug}` | detail (file key never exposed), `reviews`; optional Bearer → `owned`, `my_review` (owners also see unpublished ebooks) |
| PUT / DELETE | `/api/ebooks/{slug}/review` | same as course reviews, for buyers |
| GET | `/api/ebooks/{slug}/file?disposition=inline\|attachment` | Bearer, owners only → `{url, expires_at}`: `/media/{token}` for uploaded Telegram media, else a presigned (1 h) S3/R2 link; 403 not bought, 404 no file, 503 `storage_unavailable` |
| POST | `/api/auth/register` | `{full_name, phone, password}` → 201 `{token, max_age, user}`; 409 `phone_taken`, 422 `invalid_phone` / `invalid_name` / `weak_password` |
| POST | `/api/auth/login` | `{phone, password}` → `{token, max_age, user}`; 401 `invalid_credentials` |
| POST | `/api/auth/logout` | Bearer → 204, deletes the session |
| GET | `/api/me` | Bearer → user; 401 `unauthorized` |
| PATCH | `/api/me` | Bearer, `{full_name, email}` (empty email clears it) → user; 409 `email_taken`, 422 `invalid_email` / `invalid_name` |
| POST | `/api/me/password` | Bearer, `{current_password, new_password}` → 204, signs out other sessions; 422 `wrong_password` / `weak_password` |
| GET | `/api/me/orders` | Bearer → own orders with items, newest first |
| GET | `/api/me/library` | Bearer → `{courses, ebooks}` the user owns (card shape; courses add `lesson_count`, `completed_lessons`) |
| GET | `/api/recommendations` | optional Bearer → `{courses, ebooks, personalized}`: 8 unowned published items each, ranked by co-purchase, then shared category, then popularity (signed out: popularity) |
| GET | `/api/cart` | Bearer → `{items, total_cents}` (published items; total uses sale price) |
| POST | `/api/cart` | Bearer, `{course_id}` or `{ebook_id}` → cart; 404, 409 `already_owned`, 422 `invalid_item` |
| DELETE | `/api/cart/{id}` | Bearer → cart |
| GET | `/api/payment-methods` | active methods with QR image and account |
| POST | `/api/orders` | Bearer, `{payment_method}` → 201 order (cart → `pending_payment`; free → `paid` + granted); 422 `cart_empty` / `invalid_payment_method` |
| GET | `/api/orders/{id}` | Bearer, own orders only → order with items + payment method |
| POST | `/api/orders/{id}/submit` | Bearer, `{payment_ref}` → `awaiting_approval` (from `pending_payment` or `rejected`); 409 `invalid_status` |
| POST | `/api/orders/{id}/proof` | Bearer, own order in `pending_payment` / `awaiting_approval` / `rejected`; multipart `size` then `file` (PNG/JPEG/WebP ≤ 10 MB) → order with `payment_proof_url`; the screenshot is posted with the order details to `TELEGRAM_ADMIN_CHAT_ID`; 409 `invalid_status`, 422 `invalid_image` / `file_too_large` |
| POST | `/api/orders/{id}/cancel` | Bearer → `cancelled` (from `pending_payment` only) |
| GET | `/api/admin/orders?status=&q=&page=&per_page=` | admin → paged orders with buyer + items; `awaiting_approval` oldest first; 403 `forbidden` |
| POST | `/api/admin/orders/{id}/approve` | admin → `paid` + grants enrollments / ebooks (idempotent; also from `pending_payment` / `rejected`) |
| POST | `/api/admin/orders/{id}/reject` | admin, `{note}` → `rejected` (from `awaiting_approval`); user sees the note and can resubmit |
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
| POST | `/api/telegram/webhook` | Telegram only (`X-Telegram-Bot-Api-Secret-Token` = `TELEGRAM_WEBHOOK_SECRET`, else 401): videos/documents posted in the storage channel are imported as media (idempotent) |
| POST | `/api/admin/media` | admin, multipart `size` then `file` → 201 media; the file is streamed to the Telegram storage channel (≤ 20 MB public Bot API, ≤ 2000 MB with a `--local` server); 422 `invalid_size` / `file_too_large` / `missing_file`, 503 `telegram_unavailable` / `telegram_failed`. Lessons / ebooks take `media_id`, which wins over `video_id` / `file_key` |
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
