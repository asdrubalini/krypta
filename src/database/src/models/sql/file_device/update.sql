UPDATE `file_device`
SET `file_device`.`is_unlocked` = ?,
    `file_device`.`is_encrypted` = ?
WHERE 
    `file_device`.`file_id` = ? AND 
    `file_device`.`device_id` = ?
RETURNING `file_id`, `device_id`, `is_unlocked`, `is_encrypted`;
