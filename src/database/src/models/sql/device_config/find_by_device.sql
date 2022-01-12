SELECT `locked_path`
FROM `device_config`
JOIN `device` ON `device`.`id` = `device_config`.`id`
WHERE `device`.`device_id` = ?;
