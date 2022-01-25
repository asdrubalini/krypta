BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS `file_tag` (
	`file_id` INTEGER NOT NULL,
	`tag_id` INTEGER NOT NULL
) STRICT;

CREATE TABLE IF NOT EXISTS `tag` (
	`id` INTEGER NOT NULL UNIQUE,
	`name` TEXT NOT NULL UNIQUE,
	PRIMARY KEY(`id` AUTOINCREMENT)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS `tag_name` ON `tag` (`name`);

CREATE TABLE IF NOT EXISTS `file` (
	`id` INTEGER NOT NULL UNIQUE,
	`title` TEXT NOT NULL UNIQUE,
	`path` TEXT NOT NULL UNIQUE,
	`random_hash` TEXT NOT NULL UNIQUE,
	`contents_hash` TEXT NOT NULL,
	`size` INTEGER NOT NULL,
	`created_at` TEXT NOT NULL,
	`updated_at` TEXT NOT NULL,
	`key` BLOB NOT NULL,
	`nonce` BLOB NOT NULL,
	PRIMARY KEY(`id` AUTOINCREMENT)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS `file_title` ON `file` (`title` ASC);
CREATE UNIQUE INDEX IF NOT EXISTS `file_random_hash` ON `file` (`random_hash`);
CREATE UNIQUE INDEX IF NOT EXISTS `file_time` ON `file` (`created_at`, `updated_at`);
CREATE UNIQUE INDEX IF NOT EXISTS `file_path` ON `file` (`path`);

CREATE TABLE IF NOT EXISTS `file_device` (
	`file_id` INTEGER NOT NULL,
	`device_id` INTEGER NOT NULL,
	`is_unlocked` INTEGER NOT NULL,
	`is_locked` INTEGER NOT NULL,
	`last_modified` REAL NOT NULL,
	PRIMARY KEY(`file_id`, `device_id`)
) STRICT;

CREATE TABLE IF NOT EXISTS `device` (
	`id` INTEGER NOT NULL UNIQUE,
	`platform_id` TEXT NOT NULL UNIQUE,
	`name` TEXT NOT NULL UNIQUE,
	PRIMARY KEY(`id` AUTOINCREMENT)
) STRICT;

CREATE TABLE IF NOT EXISTS `device_config` (
	`id` INTEGER NOT NULL UNIQUE,
	`device_id` INTEGER NOT NULL,
	`locked_path` TEXT,
	`unlocked_path` TEXT,
	PRIMARY KEY(`id` AUTOINCREMENT)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS `device_config_device_id` ON `device_config` (`device_id`);

COMMIT;
