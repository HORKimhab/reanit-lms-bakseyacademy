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
cp .env.example .env           # adjust DATABASE_URL if needed
cargo run                      # http://127.0.0.1:8080/api/health

# 3. demo data (dev only — truncates tables!)
psql postgres://localhost/reanit_lms -f seeds/dev.sql

# 4. web
cd ../frontend
cp .env.example .env
bun install
bun run dev                    # http://localhost:5173
```

## API

Authenticated routes take `Authorization: Bearer <token>` (the token from register/login).

| Method | Path | Notes |
|---|---|---|
| GET | `/api/health` | DB ping |
| GET | `/api/home` | published banners + 8 newest courses + ebooks |
| GET | `/api/categories` | |
| GET | `/api/courses?q=&category=&page=&per_page=` | paged, search title/instructor |
| GET | `/api/courses/{slug}` | detail + curriculum (video id only for preview lessons) |
| GET | `/api/ebooks?q=&category=&page=&per_page=` | paged, search title/author |
| GET | `/api/ebooks/{slug}` | detail (file key never exposed) |
| POST | `/api/auth/register` | `{full_name, phone, password}` → 201 `{token, max_age, user}`; 409 `phone_taken`, 422 `invalid_phone` / `invalid_name` / `weak_password` |
| POST | `/api/auth/login` | `{phone, password}` → `{token, max_age, user}`; 401 `invalid_credentials` |
| POST | `/api/auth/logout` | Bearer → 204, deletes the session |
| GET | `/api/me` | Bearer → user; 401 `unauthorized` |
| PATCH | `/api/me` | Bearer, `{full_name, email}` (empty email clears it) → user; 409 `email_taken`, 422 `invalid_email` / `invalid_name` |
| POST | `/api/me/password` | Bearer, `{current_password, new_password}` → 204, signs out other sessions; 422 `wrong_password` / `weak_password` |
| GET | `/api/cart` | Bearer → `{items, total_cents}` (published items; total uses sale price) |
| POST | `/api/cart` | Bearer, `{course_id}` or `{ebook_id}` → cart; 404, 409 `already_owned`, 422 `invalid_item` |
| DELETE | `/api/cart/{id}` | Bearer → cart |

## Roadmap

Task-level breakdown: [`TODO.md`](TODO.md).

1. ~~Catalog: schema, courses, ebooks, categories, ratings~~
2. ~~Auth (phone + argon2, session cookies), account page~~
3. Cart → orders → ABA PayWay / Bakong KHQR webhook → enrollments
4. Lesson player (Bunny/Cloudflare Stream signed URLs), progress tracking, ebook signed downloads
5. Reviews write API, search upgrade (Meilisearch), "for you" recommendations, admin
6. i18n (Paraglide: km/en)
