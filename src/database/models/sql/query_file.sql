SELECT 
  file.id,
  file.title, 
  file.path,
  file.random_hash,
  file.data_hash
FROM 
  file_tag 
  JOIN tag ON tag.id = file_tag.tag_id 
  JOIN file ON file.id = file_tag.file_id 
WHERE 
  file.title LIKE $1
