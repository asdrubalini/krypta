SELECT `id`,
    `platform_id`,
    `name`
FROM `device`
WHERE `device`.`platform_id` LIKE :platform_id;
