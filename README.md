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
| GET | `/api/media/{token}` | streams a Telegram-stored file; `token` (from the lesson / ebook-file endpoints) is HMAC-signed with `MEDIA_SIGNING_KEY` and expires (4 h video, 1 h files); honors `Range` → 206 / 416; 404 for bad or expired tokens |
| GET | `/api/admin/media?q=&page=&per_page=` | admin → paged media library (newest first, `q` searches file names) |
| POST | `/api/telegram/webhook` | Telegram only (`X-Telegram-Bot-Api-Secret-Token` = `TELEGRAM_WEBHOOK_SECRET`, else 401): videos/documents posted in the storage channel are imported as media (idempotent) |
| POST | `/api/admin/media` | admin, multipart `size` then `file` → 201 media; the file is streamed to the Telegram storage channel (≤ 50 MB public Bot API, ≤ 2000 MB with a `--local` server); 422 `invalid_size` / `file_too_large` / `missing_file`, 503 `telegram_unavailable` / `telegram_failed`. Lessons / ebooks take `media_id`, which wins over `video_id` / `file_key` |
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
