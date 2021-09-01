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
- `is_remote`: whether or not the file exists on remote endpoint
- `is_encrypted`: whether or not the file has been encrypted
- `random_hash`: randomly-generated hash unique to each file (SHA-256)
- `created_at`
- `updated_at`

Another crucial table is `endpoint` which stores details about configured enpoints, such as connection method and credentials.

Finally, the `key` table stores the encryption key.

TODO: add a log table

Other tables are used to store tags and link them to files.

## File states

Each single file can be in three states:
- (1) Not present in the database
- (2) Present in the database, in an unencrypted format and not on the cloud
- (3) Present in the database, in an encrypted format but not on the cloud
- (4) Present in the database, in and encrypted format and on the cloud

You can know in what state each file is by executing an SQL query:

`SELECT is_remote, is_encrypted FROM file WHERE random_hash = ?`

if it returns `null`, then the file is in the (1) state. If it returns `(0, 0)`, then the file is in the (2) state. If it returns `(0, 1)`, then the file is in the (3) state and finally, if it returns `(1, 1)` we are in the (4) state.

## Local storage

It is described as `local storage` a folder on the host machine which contains encrypted files. Note that for the tool to work, it is not necessary that the local storage is up-to-date with the cloud. There can be files that are only in the local storage, and files that are only on the cloud. 

A file that is only in the local storage must be synchronized.
A file that is only on the cloud can be downloaded if requested.

Note that the database always has all files and keeps track of which ones are present on the remote host and which ones are present in the local storage.
