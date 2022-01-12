INSERT OR IGNORE INTO `device_config`(`device_id`, `locked_path`, `unlocked_path`) 
VALUES (?, NULL, NULL)
RETURNING `id`, `device_id`, `locked_path`, `unlocked_path`;
