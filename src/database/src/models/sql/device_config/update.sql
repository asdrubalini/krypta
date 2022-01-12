UPDATE `device_config`
SET `device_id` = ?,
    `locked_path` = ?,
    `unlocked_path` = ?
WHERE `id` = ?
RETURNING `id`, `device_id`, `locked_path`, `unlocked_path`;
