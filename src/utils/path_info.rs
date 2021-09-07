use std::{iter::FromIterator, path::PathBuf, sync::Arc};

use tokio::{fs::File, sync::Semaphore};

/// Holds all the information we have about a path and its details
#[derive(Clone)]
pub struct PathInfo {
    pub path: PathBuf,
    pub size: Option<u64>,
}

impl PathInfo {
    /// Retrieve self.size or get default value in case it is not available
    pub fn size_or_default(&self) -> u64 {
        self.size.unwrap_or(0)
    }

    /// Try fs access and update self.size with path's file size if possible
    async fn try_update_size(&mut self, full_path: PathBuf) -> Option<()> {
        if self.size.is_some() {
            return None;
        }

        let file = File::open(full_path).await.ok()?;
        let size: u64 = file.metadata().await.ok()?.len();

        self.size = Some(size);

        Some(())
    }
}

impl From<&PathBuf> for PathInfo {
    /// Build a PathInfo from a PathBuf, with empty fields
    fn from(path: &PathBuf) -> Self {
        Self {
            path: path.to_owned(),
            size: None,
        }
    }
}

/// Holds a collection of paths together with their info, if available
pub struct PathInfos {
    pub paths: Vec<PathInfo>,
}

impl PathInfos {
    /// Try to populate the empty fields in PathInfo(s), returning a new copy
    pub async fn try_populate_all(self, prefix: PathBuf) -> Self {
        // Use a semaphore in order not to exceed os's max file open count
        let semaphore = Arc::new(Semaphore::new(128));
        let mut handles = Vec::new();

        for path_info in self.paths {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let mut full_path = prefix.clone();

            let handle = tokio::spawn(async move {
                full_path.push(&path_info.path);

                let mut path_info = path_info.clone();
                path_info.try_update_size(full_path).await;

                drop(permit);

                path_info
            });

            handles.push(handle);
        }

        // New paths Vec that will be populated with updated details
        let mut paths = Vec::new();

        // Wait for all tasks
        for handle in handles {
            paths.push(handle.await.unwrap());
        }

        Self { paths }
    }
}

impl FromIterator<PathInfo> for PathInfos {
    fn from_iter<T: IntoIterator<Item = PathInfo>>(paths_iter: T) -> Self {
        let paths: Vec<PathInfo> = paths_iter.into_iter().collect();

        Self { paths }
    }
}
