SELECT id, platform_id, name
FROM device
WHERE platform_id LIKE :platform_id;
