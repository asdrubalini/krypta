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
	`locked_hash` TEXT NOT NULL,
	`contents_hash` TEXT NOT NULL,
	`size` INTEGER NOT NULL,
	`created_at` TEXT NOT NULL,
	`updated_at` TEXT NOT NULL,
	`key` BLOB NOT NULL,
	`nonce` BLOB NOT NULL,
	PRIMARY KEY(`id` AUTOINCREMENT)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS `file_title` ON `file` (`title` ASC);
CREATE INDEX IF NOT EXISTS `file_locked_hash` ON `file` (`locked_hash`);
CREATE UNIQUE INDEX IF NOT EXISTS `file_time` ON `file` (`created_at`, `updated_at`);
CREATE UNIQUE INDEX IF NOT EXISTS `file_path` ON `file` (`path`);

COMMIT;
