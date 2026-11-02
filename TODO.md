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
- [ ] F1 Paraglide km/en + header language switcher

## Later
- [ ] Automatic payment confirmation: Bakong KHQR dynamic QR + status check, ABA PayWay checkout + pushback webhook (same grant path as admin approval)
- [ ] Payment screenshot upload (needs object storage)
- [ ] Self-service password reset (SMS OTP)
- [ ] Google sign-in (server-side OAuth; `users.google_sub`)
