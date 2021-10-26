# Major TODOs
- Working implementation of SHA-256 file hasing both in single and bulk mode + tests
- Evaluate Anyhow
- Accept AsRef<Path> on api
- Use a generic Bulkable implementation in crypto in order to implement bulk actions
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

