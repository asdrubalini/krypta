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
- `random_hash`: randomly-generated hash unique to each file (SHA-256)
- `data_hash`: original data hash, used to make sure that decryption was successful (CRC-32)
- `created_at`
- `updated_at`

Another crucial table is `endpoint` which stores details about configured enpoints, such as connection method and credentials.

Finally, the `key` table stores the encryption key.

Other tables are used to store tags and link them to files.
