# Features

- Encrypted file storage on cloud providers that support folder sync
- Hidden filenames
- Fast file search via indexed SQLite3 database
- File tagging

# Description

## Database file

There is one main database file which stores unencrypted filenames, tags, hashes and other details. This file shall not be uploaded to the cloud provider as it contains critical information on the files. It also contains the decryption key used to unlock remote files.

The main table in the database file, `file`, stores all the files, their hash, path and friendly name.
It has the following fields:
- `id`: database's incremental and unique id
- `title`: friendly file's name
- `path`: original, unencrypted file path
- `random_hash`: randomly-generated hash unique to each file
- `contents_hash`: hash of the plaintext version
- `size`: file size, in bytes
- `created_at`
- `updated_at`

Another crucial table is `endpoint` which stores details about configured enpoints, such as connection method and credentials.

Finally, the `key` table stores the encryption key.

TODO: add a log table

Other tables are used to store tags and link them to files.

## Local storage

It is described as `local storage` a folder on the host machine which contains encrypted files. Note that for the tool to work, it is not necessary that the local storage is up-to-date with the cloud. There can be files that are only in the local storage, and files that are only on the cloud. 

A file that is only in the local storage must be synchronized.
A file that is only on the cloud can be downloaded if requested.

Note that the database always has all files and keeps track of which ones are present on the remote host and which ones are present in the local storage.

# Commands

## Init database

`krypta init <source_path>`

## Sync database

`krypta sync <source_path>`
