# TODO

Feature parity with a typical Khmer course + ebook store (reference: bakseyacademy.com — features and layout only; no copied branding, content or contact details). One task = one commit; run the checks in `CLAUDE.md` before each commit and tick the box in the same commit.

## A. Layout
- [ ] A1 Footer: address, phones, email and social links (Facebook / Telegram / YouTube) from one config, `frontend/src/lib/site.ts` (placeholders until real details exist)
- [ ] A2 Homepage slideshow: `banners` table + `GET /api/banners`, `Carousel.svelte` replaces the gradient hero

## B. Accounts (roadmap 2)
Open question: phone (+855) + password as the login, email optional?
- [ ] B1 Migration: `users.phone` (unique, E.164), `email` nullable, `sessions` table
- [ ] B2 `POST /api/auth/register` (name, phone, password; argon2); `AppError::Conflict`, `AppError::Unauthorized`
- [ ] B3 `POST /api/auth/login`, `POST /api/auth/logout`, `GET /api/me` (HttpOnly session cookie)
- [ ] B4 `/auth` page (login / register tabs, +855 prefix); `hooks.server.ts` forwards the cookie through `api()`, `locals.user`
- [ ] B5 Header account menu + logout; `/account` (profile, change password)
- [ ] B6 Forgot password: Telegram support link first, SMS/OTP reset later

## C. Commerce (roadmap 3)
Open question: Bakong KHQR or ABA PayWay first?
- [ ] C1 Migration: `cart_items`, `orders`, `order_items` (price snapshot in cents), `enrollments`, `ebook_purchases`
- [ ] C2 Cart API (`GET/POST/DELETE /api/cart`) + header cart icon with count
- [ ] C3 `/cart` page + checkout creating a `pending` order
- [ ] C4 Payment QR + webhook: mark order `paid`, grant enrollment / ebook (idempotent)
- [ ] C5 `/account/orders`, "My courses / My ebooks", "Continue learning" on owned items

## D. Learning (roadmap 4)
- [ ] D1 `GET /api/courses/{slug}/lessons/{id}`: signed video URL for enrolled users only
- [ ] D2 Lesson player page + curriculum sidebar
- [ ] D3 Progress tracking (`lesson_progress`, mark complete, % on cards)
- [ ] D4 Ebook signed download / reader for owners

## E. Discovery & admin (roadmap 5)
- [ ] E1 Reviews write API (owners only) + form on detail pages
- [ ] E2 `/for-you` recommendations (same category + co-purchase, server-side)
- [ ] E3 Admin CRUD: courses, sections, lessons, ebooks, banners, categories, publish toggle
- [ ] E4 Search upgrade (Postgres trigram first, Meilisearch if needed)

## F. i18n (roadmap 6)
- [ ] F1 Paraglide km/en + header language switcher

## Later
- [ ] Google sign-in (server-side OAuth; `users.google_sub`)
