# Major TODOs
- Fuly implement init command
- Decide if sync should detect deleted files (probably not, deletion should happen manually)
- Switch to anyhow in the main crate
- Mechanism than can detect file changes based on file's date and time modification (and then fallback to SHA-256)
- Add file's fs last modification time in database schema
- Add file's SHA-256 in database schema
- Use file encryption from implementationw
- Stable cli interface that can add, remove and search files

# Long term TODOs
- Revisit all async traits and remove the #[async_trait] macro

# Minor TODOs
- Bring some true TUI
- Write a lot more tests
- Implement tags and show them on the TUI
- Implement proper upload mechanism with S3 based API
