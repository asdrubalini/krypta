UPDATE file
SET title = :title,
    path = :path,
    locked_hash = :locked_hash,
    contents_hash = :contents_hash,
    size = :size,
    created_at = :created_at,
    updated_at = :updated_at,
    key = :key,
    nonce = :nonce
WHERE id = :id RETURNING *;
