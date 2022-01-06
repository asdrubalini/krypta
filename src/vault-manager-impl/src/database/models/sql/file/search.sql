SELECT title,
  path,
  random_hash,
  contents_hash,
  size,
  created_at,
  updated_at
FROM file
WHERE file.title LIKE ?