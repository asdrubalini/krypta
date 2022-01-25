use std::{collections::HashMap, path::PathBuf};

use database::{models, traits::FetchAll, Database};

// TODO: consider swithing to OsString

#[derive(Debug)]
struct PathTree {
    paths: PathKind,
}

impl Default for PathTree {
    fn default() -> Self {
        Self {
            paths: PathKind::root(),
        }
    }
}

impl FromIterator<PathBuf> for PathTree {
    fn from_iter<T: IntoIterator<Item = PathBuf>>(iter: T) -> Self {
        let mut tree = PathTree::default();

        for file_path in iter {
            tree.add_file(file_path);
        }

        tree
    }
}

impl PathTree {
    pub fn add_file(&mut self, file_path: PathBuf) {
        let mut current_path = match &mut self.paths {
            PathKind::Directory { contents: content } => content,
            PathKind::File => panic!("ciao"),
        };
        let path_len = file_path.iter().count();

        for (i, piece) in file_path.iter().enumerate() {
            let piece = piece.to_string_lossy().to_string();

            if current_path.get(&piece).is_some() {
                // path already exists, just traverse
                current_path = match current_path.get_mut(&piece).unwrap() {
                    PathKind::Directory { contents: content } => content,
                    PathKind::File => panic!("ciao"),
                };

                continue;
            }

            // println!("inserting {piece}");

            if i + 1 == path_len {
                // Current piece is a file
                current_path.insert(piece.clone(), PathKind::File);
            } else {
                // Current piece is a directory
                current_path.insert(
                    piece.clone(),
                    PathKind::Directory {
                        contents: HashMap::default(),
                    },
                );

                current_path = match current_path.get_mut(&piece).unwrap() {
                    PathKind::Directory { contents: content } => content,
                    PathKind::File => panic!("hehe"),
                };
            }
        }
    }
}

impl Iterator for PathTree {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

#[derive(Debug)]
enum PathKind {
    Directory { contents: HashMap<String, PathKind> },
    File,
}

impl PathKind {
    fn root() -> Self {
        PathKind::Directory {
            contents: HashMap::default(),
        }
    }
}

pub async fn tree(db: &mut Database) -> anyhow::Result<()> {
    let tree: PathTree = models::File::fetch_all(db)?
        .into_iter()
        .map(|f| f.as_path_buf())
        .collect();

    println!("{tree:#?}");

    Ok(())
}
