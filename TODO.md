# Major TODOs
- files may have changed, so we should check the last modified date
to make sure that they have not, or we don't try to detect them at all
and instead rely on the user with a special `add` command or something like that.
NOTE that its not clear where the last os modification date should be stored, in a context
where multiple devices can operate on a single shared database.
Now this distinction does not exist, and a single database can only be modified
by a single device (due to having fields like `is_remote` and `is_encrypted`)
Maybe there can be a separated table specific to each device with `is_remote`, `is_encrypted`, and 
something like `fs_last_modified_at`.

- Add file's fs last modification time in database schema

- Actually encrypt files and update db accordingly
- Accept encrypted folder as args

- Decide if sync should detect deleted files (probably not, deletion should happen manually)
- Switch to anyhow in the main crate
- Use file encryption from implementationw
- Stable cli interface that can add, remove and search files

# Long term TODOs
- Revisit all async traits and remove the #[async_trait] macro

# Minor TODOs
- Bring some true TUI
- Write a lot more tests
- Implement tags and show them on the TUI
- Implement proper upload mechanism with S3 based API
