SELECT `device_config`.*
FROM `device_config`
INNER JOIN `device` ON `device`.`id` = `device_config`.`device_id`
WHERE `device`.`id` = ?;
