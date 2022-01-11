INSERT INTO file (
    title,
    path,
    random_hash,
    contents_hash,
    size,
    created_at,
    updated_at,
    key,
    nonce
  )
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
RETURNING id,
  title,
  path,
  random_hash,
  contents_hash,
  size,
  created_at,
  updated_at,
  key,
  nonce