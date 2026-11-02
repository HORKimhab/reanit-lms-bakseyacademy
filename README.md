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

| Method | Path | Notes |
|---|---|---|
| GET | `/api/health` | DB ping |
| GET | `/api/home` | 8 newest courses + ebooks |
| GET | `/api/categories` | |
| GET | `/api/courses?q=&category=&page=&per_page=` | paged, search title/instructor |
| GET | `/api/courses/{slug}` | detail + curriculum (video id only for preview lessons) |
| GET | `/api/ebooks?q=&category=&page=&per_page=` | paged, search title/author |
| GET | `/api/ebooks/{slug}` | detail (file key never exposed) |

## Roadmap

Task-level breakdown: [`TODO.md`](TODO.md).

1. ~~Catalog: schema, courses, ebooks, categories, ratings~~
2. Auth (argon2 + session cookies), account page
3. Cart → orders → ABA PayWay / Bakong KHQR webhook → enrollments
4. Lesson player (Bunny/Cloudflare Stream signed URLs), progress tracking, ebook signed downloads
5. Reviews write API, search upgrade (Meilisearch), "for you" recommendations, admin
6. i18n (Paraglide: km/en)
