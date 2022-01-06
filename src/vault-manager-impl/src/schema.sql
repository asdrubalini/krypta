BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS "file_tag" (
	"file_id" INTEGER NOT NULL,
	"tag_id" INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS "tag" (
	"id" INTEGER NOT NULL UNIQUE,
	"name" TEXT NOT NULL UNIQUE,
	PRIMARY KEY("id" AUTOINCREMENT)
);

CREATE TABLE IF NOT EXISTS "file" (
	"id" INTEGER NOT NULL UNIQUE,
	"title" TEXT NOT NULL UNIQUE,
	"path" TEXT NOT NULL UNIQUE,
	"is_remote" INTEGER NOT NULL,
	"is_encrypted" INTEGER NOT NULL,
	"random_hash" TEXT NOT NULL UNIQUE,
	"contents_hash" TEXT NOT NULL,
	"size" BLOB NOT NULL,
	"created_at" TEXT NOT NULL,
	"updated_at" TEXT NOT NULL,
	PRIMARY KEY("id" AUTOINCREMENT)
);

CREATE TABLE IF NOT EXISTS "vault_info" (
	"name" TEXT NOT NULL,
	"total_size" BLOB NOT NULL
);

CREATE TABLE IF NOT EXISTS "endpoint" (
	"id" INTEGER NOT NULL UNIQUE,
	PRIMARY KEY("id" AUTOINCREMENT)
);

CREATE TABLE IF NOT EXISTS "key" (
	"id" INTEGER NOT NULL UNIQUE,
	PRIMARY KEY("id" AUTOINCREMENT)
);

CREATE UNIQUE INDEX IF NOT EXISTS "tag_name" ON "tag" ("name");

CREATE UNIQUE INDEX IF NOT EXISTS "file_title" ON "file" ("title" ASC);

CREATE UNIQUE INDEX IF NOT EXISTS "file_random_hash" ON "file" ("random_hash");

CREATE UNIQUE INDEX IF NOT EXISTS "file_time" ON "file" ("created_at", "updated_at");

CREATE INDEX IF NOT EXISTS "file_remote_encrypted_flags" ON "file" ("is_remote", "is_encrypted");

COMMIT;