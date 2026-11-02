# Reanit LMS

Khmer-first online course + ebook store. `backend/` is a Rust API (Axum 0.8, SQLx 0.8, Postgres); `frontend/` is SvelteKit (Svelte 5 runes, Tailwind v4, adapter-node). Setup, run commands, the API table and the roadmap live in `README.md`; keep the API table there current when you add or change a route.

## Verify

- Backend: `cd backend && cargo check` (and `cargo clippy` when available). No test suite yet.
- Frontend: `cd frontend && bun run check` (svelte-check). Bun is the package manager.
- `seeds/dev.sql` truncates every catalog and user table; run it only against a local dev database.

## Backend conventions

- Routes are registered in `src/routes/mod.rs`, one module per resource, all under `/api`.
- Handlers return `AppResult<Json<T>>` (`src/error.rs`). Errors serialize as `{"error": "<code>"}`; add a new `AppError` variant for a new error class instead of building ad-hoc responses.
- Queries use runtime `sqlx::query_as::<_, T>(...)` with `.bind(...)`, not the compile-time `query!` macros (no `DATABASE_URL` / offline data needed to build). SQL fragments shared between list/count/detail live in `const` strings (see `CARD_SELECT` / `FILTER` in `routes/courses.rs`).
- Run independent queries concurrently with `tokio::try_join!`.
- Response structs live in `src/models.rs`; detail responses compose a card with `#[serde(flatten)]`.
- Migrations live in `backend/migrations/NNNN_name.sql` and are applied automatically on `cargo run`. Add a new numbered file; leave applied migrations untouched.
- Money is integer US cents (`price_cents`, `sale_price_cents`); `sale_price_cents < price_cents` is enforced in the schema.
- Public endpoints only return `published` rows.

## Access-control invariants

Paid content stays server-side until enrollment exists (roadmap step 3–4):
- Lesson `video_id` is exposed only as `preview_video_id` on `is_preview` lessons; playback goes through `GET /api/courses/{slug}/lessons/{id}`, which checks enrollment and returns a signed URL (`src/video.rs`).
- Ebook file keys are never serialized; owners get a short-lived presigned URL from `GET /api/ebooks/{slug}/file` (`src/storage.rs`).
Admin endpoints (`/api/admin/*`, `AdminUser`) may return lesson `video_id`, but ebook `file_key` stays write-only there too (`has_file`).
Preserve these when touching course/ebook responses.

## Frontend conventions

- All API calls go through `api()` in `src/lib/server/api.ts`, from `+page.server.ts` / `+layout.server.ts` loads only. The browser never talks to the Rust API directly; `API_URL` is a private env var.
- `src/lib/types.ts` mirrors the Rust response structs by hand; update both sides together.
- Components use Svelte 5 runes (`$props()`, `$state`, `$derived`).
- UI copy is Khmer. Format numbers, prices and durations through `src/lib/format.ts` (`khmerNumber`, `formatPrice`, `formatDuration`), which renders Khmer digits and units.
- Styling is Tailwind v4 configured in `src/app.css` via `@theme` (brand colors, Noto Sans Khmer); there is no `tailwind.config.js`. Keep the generous body line-height, which Khmer subscripts need.
