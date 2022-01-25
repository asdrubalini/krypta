use std::{collections::HashMap, path::PathBuf};

use database::{models, traits::FetchAll, Database};

// TODO: consider swithing to OsString

#[derive(Default, Debug)]
struct PathTree {
    paths: PathKind,
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
                // Already exists
                continue;
            }

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

impl Default for PathKind {
    fn default() -> Self {
        PathKind::Directory {
            contents: HashMap::default(),
        }
    }
}

pub async fn tree(db: &mut Database) -> anyhow::Result<()> {
    let files = models::File::fetch_all(db)?
        .into_iter()
        .map(|file| file.as_path_buf());

    let mut tree = PathTree::default();

    for file_path in files {
        tree.add_file(file_path);
    }

    println!("{tree:#?}");

    Ok(())
}
