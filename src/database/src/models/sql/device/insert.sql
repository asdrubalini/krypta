INSERT INTO device (platform_id, name)
VALUES (?, ?)
RETURNING id,
    platform_id,
    name