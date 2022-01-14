UPDATE `file`
SET `title` = ?,
    `path` = ?,
    `random_hash` = ?,
    `contents_hash` = ?,
    `size` = ?,
    `created_at` = ?,
    `updated_at` = ?,
    `key` = ?,
    `nonce` = ?
WHERE 
    `id` = ?
RETURNING *;
