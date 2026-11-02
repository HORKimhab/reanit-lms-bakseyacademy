-- DEV ONLY: wipes catalog + users and inserts placeholder data.
-- Run after the API has applied migrations:  psql "$DATABASE_URL" -f seeds/dev.sql
BEGIN;

TRUNCATE lesson_progress, enrollments, ebook_purchases, order_items, orders, cart_items, payment_methods, sessions, banners, reviews, lessons, course_sections, courses, ebooks, categories, instructors, users CASCADE;

-- Demo users have no password; register a new account to log in.
INSERT INTO users (phone, email, full_name) VALUES
    ('+85510000001', 'student1@example.com', 'Student One'),
    ('+85510000002', 'student2@example.com', 'Student Two'),
    ('+85510000003', NULL,                   'Student Three');

INSERT INTO categories (slug, name_km, name_en, position) VALUES
    ('technology',  'បច្ចេកវិទ្យា', 'Technology',  1),
    ('marketing',   'ទីផ្សារ',      'Marketing',   2),
    ('business',    'អាជីវកម្ម',    'Business',    3),
    ('investment',  'វិនិយោគ',      'Investment',  4),
    ('real-estate', 'អចលនទ្រព្យ',   'Real estate', 5);

INSERT INTO instructors (name, bio) VALUES
    ('សុខ ដារ៉ា',   'Demo instructor — software engineer.'),
    ('ចាន់ សុភា',   'Demo instructor — digital marketer.'),
    ('លី វិចិត្រ', 'Demo instructor — investor.');

INSERT INTO courses (slug, title, summary, description, instructor_id, category_id, price_cents, sale_price_cents, published, created_at)
SELECT v.slug, v.title, v.summary, v.description,
       (SELECT id FROM instructors WHERE name = v.instructor),
       (SELECT id FROM categories WHERE slug = v.category),
       v.price, v.sale, true, now() - v.age
FROM (VALUES
    ('rust-for-beginners',  'មូលដ្ឋានគ្រឹះ Rust សម្រាប់អ្នកចាប់ផ្តើម', 'រៀន Rust ពីសូន្យ', E'វគ្គនេះនឹងបង្រៀនអ្នកពី Rust។\nសមស្របសម្រាប់អ្នកចាប់ផ្តើម។', 'សុខ ដារ៉ា', 'technology', 3000, 1500, interval '1 day'),
    ('web-with-svelte',     'បង្កើតគេហទំព័រជាមួយ SvelteKit',          'Frontend សម័យថ្មី',   'Placeholder description.', 'សុខ ដារ៉ា', 'technology', 4000, NULL, interval '2 days'),
    ('facebook-marketing',  'ទីផ្សារឌីជីថលលើ Facebook',               'ផ្សព្វផ្សាយឲ្យមានប្រសិទ្ធភាព', 'Placeholder description.', 'ចាន់ សុភា', 'marketing', 5000, 2500, interval '3 days'),
    ('small-business-101',  'ចាប់ផ្តើមអាជីវកម្មខ្នាតតូច',               'ពីគំនិតទៅជាអាជីវកម្ម', 'Placeholder description.', 'ចាន់ សុភា', 'business', 3500, NULL, interval '4 days'),
    ('stock-investing',     'ការវិនិយោគភាគហ៊ុនសម្រាប់អ្នកចាប់ផ្តើម',    'វិនិយោគដោយប្រុងប្រយ័ត្ន', 'Placeholder description.', 'លី វិចិត្រ', 'investment', 6000, 4500, interval '5 days'),
    ('real-estate-basics',  'មូលដ្ឋានអចលនទ្រព្យ',                     'ទិញ លក់ ជួល',        'Placeholder description.', 'លី វិចិត្រ', 'real-estate', 6000, NULL, interval '6 days')
) AS v(slug, title, summary, description, instructor, category, price, sale, age);

INSERT INTO course_sections (course_id, title, position)
SELECT c.id, 'ផ្នែកទី ' || s || '៖ ' || (ARRAY['ការណែនាំ', 'អនុវត្តជាក់ស្តែង', 'គម្រោងបញ្ចប់'])[s], s
FROM courses c, generate_series(1, 3) AS s;

INSERT INTO lessons (section_id, title, position, duration_seconds, is_preview)
SELECT cs.id, 'មេរៀនទី ' || l, l, 300 + l * 120, (cs.position = 1 AND l = 1)
FROM course_sections cs, generate_series(1, 4) AS l;

INSERT INTO ebooks (slug, title, author_name, summary, category_id, price_cents, sale_price_cents, page_count, published, created_at)
SELECT v.slug, v.title, v.author, v.summary, (SELECT id FROM categories WHERE slug = v.category),
       v.price, v.sale, v.pages, true, now() - v.age
FROM (VALUES
    ('personal-finance', 'ហិរញ្ញវត្ថុផ្ទាល់ខ្លួន',  'លី វិចិត្រ', 'គ្រប់គ្រងលុយឲ្យបានល្អ', 'investment', 1000, 500, 120, interval '1 day'),
    ('brand-building',   'ការកសាងម៉ាកយីហោ',     'ចាន់ សុភា', 'ម៉ាកដែលអតិថិជនចងចាំ', 'marketing',  1200, NULL, 96, interval '2 days'),
    ('startup-playbook', 'សៀវភៅណែនាំ Startup',   'សុខ ដារ៉ា', 'ពីគំនិតទៅផលិតផល',  'business',   1500, 990, 150, interval '3 days')
) AS v(slug, title, author, summary, category, price, sale, pages, age);

-- Deterministic pseudo-random ratings; one course left unrated to exercise the empty state.
INSERT INTO reviews (user_id, course_id, rating, comment)
SELECT u.id, c.id, 3 + abs(hashtext(u.full_name || c.slug)) % 3, 'ល្អណាស់!'
FROM users u CROSS JOIN courses c
WHERE c.slug <> 'small-business-101';

INSERT INTO reviews (user_id, ebook_id, rating)
SELECT u.id, e.id, 3 + abs(hashtext(u.full_name || e.slug)) % 3
FROM users u CROSS JOIN ebooks e;

INSERT INTO banners (title, subtitle, link_url, position, published) VALUES
    ('រៀនជំនាញថ្មីៗ គ្រប់ពេល គ្រប់ទីកន្លែង', 'វគ្គសិក្សា និងសៀវភៅពីគ្រូជំនាញ ជាភាសាខ្មែរ។', '/courses', 1, true),
    ('សៀវភៅថ្មីៗ ប្រចាំខែ', 'អានបានគ្រប់ឧបករណ៍។', '/ebooks', 2, true),
    ('Draft banner', NULL, NULL, 3, false);

-- Placeholder QR images (frontend/static/payments); replace with the real merchant QR codes.
INSERT INTO payment_methods (code, name_km, name_en, account_name, account_number, qr_image_url, position, active) VALUES
    ('bakong_khqr', 'បាគង KHQR', 'Bakong KHQR', 'REANIT ACADEMY', 'reanit@aclb', '/payments/bakong-khqr-placeholder.svg', 1, true),
    ('aba_payway',  'ABA PayWay', 'ABA PayWay',  'REANIT ACADEMY', '000 000 000', '/payments/aba-payway-placeholder.svg',  2, true);

COMMIT;
