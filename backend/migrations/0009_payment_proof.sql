-- Payment screenshot the buyer attached (posted to the admins' Telegram chat).
ALTER TABLE orders ADD COLUMN payment_proof_media_id UUID REFERENCES media_files (id) ON DELETE SET NULL;
