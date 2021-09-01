SELECT 
  file.id,
  file.title, 
  file.path,
  file.is_remote,
  file.is_encrypted,
  file.random_hash,
  file.data_hash,
  file.created_at,
  file.updated_at
FROM 
  file
WHERE 
  file.title LIKE $1
