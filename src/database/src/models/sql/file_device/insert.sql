INSERT INTO `file_device` (`file_id`, `device_id`, `is_unlocked`, `is_locked`, `last_modified`)
VALUES (:file_id, :device_id, :is_unlocked, :is_locked, :last_modified) RETURNING *;