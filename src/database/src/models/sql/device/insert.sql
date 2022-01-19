INSERT INTO `device` (`platform_id`, `name`)
VALUES (:platform_id, :name) RETURNING *;