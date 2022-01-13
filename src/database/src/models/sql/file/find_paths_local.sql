SELECT `file`.`path`
FROM `file_device`
    INNER JOIN `file` ON `file_device`.`file_id` = `file`.`id`
    INNER JOIN `device` ON `file_device`.`device_id` = `device`.`id`
WHERE `device`.`platform_id` = ?
    AND `file_device`.`is_unlocked` = 1;
