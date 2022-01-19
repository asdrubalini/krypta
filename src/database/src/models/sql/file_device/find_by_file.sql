SELECT `file_device`.*
FROM `file_device`
INNER JOIN `file` ON `file_device`.`file_id` = `file`.`id`
WHERE `file`.`id` = :file_id;
