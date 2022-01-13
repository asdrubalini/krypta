INSERT INTO `file_device` (`file_id`, `device_id`, `is_unlocked`, `is_encrypted`, `last_modified`)
VALUES (?, ?, ?, ?, ?) RETURNING `file_id`, `device_id`, `is_unlocked`, `is_encrypted`, `last_modified`;
