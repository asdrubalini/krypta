UPDATE `file_device`
SET `is_unlocked` = ?,
    `is_encrypted` = ?,
    `last_modified` = ?
WHERE 
    `file_id` = ? AND `device_id` = ?
RETURNING `file_id`, `device_id`, `is_unlocked`, `is_encrypted`, `last_modified`;
