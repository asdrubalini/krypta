UPDATE `device_config`
SET `device_id` = :device_id,
    `locked_path` = :locked_path,
    `unlocked_path` = :unlocked_path
WHERE `id` = :id RETURNING *;