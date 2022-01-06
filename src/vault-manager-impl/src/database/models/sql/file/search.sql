SELECT
  title,
  path,
  is_remote,
  is_encrypted,
  random_hash,
  contents_hash,
  size,
  created_at,
  updated_at
FROM
  file
WHERE
  file.title LIKE ?