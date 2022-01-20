SELECT COUNT(*)
FROM file_device
INNER JOIN FILE ON file_device.file_id = file.id
INNER JOIN device ON file_device.device_id = device.id
WHERE device.platform_id = :platform_id
  AND file_device.is_locked = 1;
