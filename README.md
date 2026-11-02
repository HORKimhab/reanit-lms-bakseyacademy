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
cp .env.example .env           # adjust DATABASE_URL; BUNNY_* for lesson videos, STORAGE_* for ebook files
cargo run                      # http://127.0.0.1:8080/api/health

# 3. demo data (dev only — truncates tables!)
psql postgres://localhost/reanit_lms -f seeds/dev.sql

# 4. web
cd ../frontend
cp .env.example .env
bun install
bun run dev                    # http://localhost:5173
```

## Admin

Make an account an admin (it then sees **ផ្ទាំងគ្រប់គ្រង** in the account menu → `/admin/orders`):

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
| GET | `/api/courses?q=&category=&page=&per_page=` | paged, search title/instructor |
| GET | `/api/courses/{slug}` | detail + curriculum (video id only for preview lessons); optional Bearer → `owned`, `completed_lesson_ids` (enrolled users also see unpublished courses) |
| GET | `/api/courses/{slug}/lessons/{id}` | optional Bearer → lesson + signed Bunny Stream `video_url` (`expires_at`), `completed`; enrolled users (even if unpublished) or preview lessons; 401 signed out, 403 not enrolled |
| PUT / DELETE | `/api/courses/{slug}/lessons/{id}/complete` | Bearer → 204; mark / unmark complete (PUT needs enrollment, 403 otherwise) |
| GET | `/api/ebooks?q=&category=&page=&per_page=` | paged, search title/author |
| GET | `/api/ebooks/{slug}` | detail (file key never exposed); optional Bearer → `owned` (owners also see unpublished ebooks) |
| GET | `/api/ebooks/{slug}/file?disposition=inline\|attachment` | Bearer, owners only → `{url, expires_at}` presigned (1 h) S3/R2 link; 403 not bought, 404 no file, 503 `storage_unavailable` |
| POST | `/api/auth/register` | `{full_name, phone, password}` → 201 `{token, max_age, user}`; 409 `phone_taken`, 422 `invalid_phone` / `invalid_name` / `weak_password` |
| POST | `/api/auth/login` | `{phone, password}` → `{token, max_age, user}`; 401 `invalid_credentials` |
| POST | `/api/auth/logout` | Bearer → 204, deletes the session |
| GET | `/api/me` | Bearer → user; 401 `unauthorized` |
| PATCH | `/api/me` | Bearer, `{full_name, email}` (empty email clears it) → user; 409 `email_taken`, 422 `invalid_email` / `invalid_name` |
| POST | `/api/me/password` | Bearer, `{current_password, new_password}` → 204, signs out other sessions; 422 `wrong_password` / `weak_password` |
| GET | `/api/me/orders` | Bearer → own orders with items, newest first |
| GET | `/api/me/library` | Bearer → `{courses, ebooks}` the user owns (card shape; courses add `lesson_count`, `completed_lessons`) |
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

## Roadmap

Task-level breakdown: [`TODO.md`](TODO.md).

1. ~~Catalog: schema, courses, ebooks, categories, ratings~~
2. ~~Auth (phone + argon2, session cookies), account page~~
3. ~~Cart → orders → QR payment (Bakong KHQR / ABA PayWay) + admin approval → enrollments~~ (automatic confirmation: see TODO "Later")
4. Lesson player (Bunny/Cloudflare Stream signed URLs), progress tracking, ebook signed downloads
5. Reviews write API, search upgrade (Meilisearch), "for you" recommendations, admin
6. i18n (Paraglide: km/en)
