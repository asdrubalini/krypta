SELECT tag.* FROM file_tag
INNER JOIN tag on tag.id = file_tag.tag_id
INNER JOIN file on file.id = file_tag.file_id
WHERE file.id = :file_id;
