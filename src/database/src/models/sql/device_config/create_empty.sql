INSERT INTO device_config (device_id, locked_path, unlocked_path)
VALUES (:device_id, NULL, NULL)
RETURNING *;
