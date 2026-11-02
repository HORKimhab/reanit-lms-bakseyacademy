-- Separate course content from buyers' payment screenshots (which never belong in the media library).
ALTER TABLE media_files ADD COLUMN purpose TEXT NOT NULL DEFAULT 'content'
    CHECK (purpose IN ('content', 'payment_proof'));
UPDATE media_files SET purpose = 'payment_proof' WHERE id IN (SELECT payment_proof_media_id FROM orders WHERE payment_proof_media_id IS NOT NULL);
