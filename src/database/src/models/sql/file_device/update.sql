UPDATE `file_device`
SET `is_unlocked` = :is_unlocked,
    `is_encrypted` = :is_encrypted,
    `last_modified` = :last_modified
WHERE `file_id` = :file_id
  AND `device_id` = :device_id RETURNING *;