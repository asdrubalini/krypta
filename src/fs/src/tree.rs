use std::{
    collections::HashMap,
    ffi::OsString,
    path::{Path, PathBuf},
};

use indexmap::IndexSet;
use itertools::Itertools;

/// Where the actual paths are stored
#[derive(Debug)]
pub struct PathTree(PathType);

impl Default for PathTree {
    fn default() -> Self {
        Self(PathType::root())
    }
}

/// TODO: accept &Path instead of PathBuf
impl FromIterator<PathBuf> for PathTree {
    fn from_iter<T: IntoIterator<Item = PathBuf>>(iter: T) -> Self {
        let mut tree = PathTree::default();

        for file_path in iter {
            tree.insert_file_path(file_path);
        }

        tree
    }
}

impl PathTree {
    pub fn insert_file_path(&mut self, file_path: impl AsRef<Path>) {
        let mut current_path = match &mut self.0 {
            PathType::Directory(contents) => contents,
            PathType::File => panic!(
                "unexpected error: root is of type PathType::File instead of PathType::Directory"
            ),
        };
        let path_len = file_path.as_ref().iter().count();

        for (i, piece) in file_path.as_ref().iter().enumerate() {
            let piece = piece.to_owned();

            if current_path.get(&piece).is_some() {
                // path already exists, just traverse
                current_path = match current_path.get_mut(&piece).unwrap() {
                    PathType::Directory (contents) => contents,
                    PathType::File => panic!(
                        "unexpected error: {piece:?} is of type PathType::File instead of PathType::Directory"
                    ),
                };

                continue;
            }

            if i + 1 == path_len {
                // Current piece is a file
                current_path.insert(piece.to_owned(), PathType::File);
            } else {
                // Current piece is a directory
                current_path.insert(piece.to_owned(), PathType::Directory(HashMap::default()));

                current_path = match current_path.get_mut(&piece).unwrap() {
                    PathType::Directory (contents ) => contents,
                    PathType::File => panic!(
                        "unexpected error: {piece:?} is of type PathType::File instead of PathType::Directory"
                    ),
                };
            }
        }
    }

    /// Traverse the tree and get all the paths sorted
    pub fn paths_ordered(&self) -> Vec<PathBuf> {
        let mut output = vec![];
        traverse_paths_ordered(&self.0, vec![], &mut output);
        output
    }

    pub fn directory_structure(self) -> Vec<PathBuf> {
        let mut output = vec![];
        traverse_paths_ordered(&self.0, vec![], &mut output);

        let mut paths_ordered: IndexSet<PathBuf> = IndexSet::new();

        for file_path in output {
            let directory: PathBuf = file_path
                .iter()
                .take(file_path.iter().count() - 1)
                .collect();

            for len in 1..directory.iter().count() + 1 {
                let partial_dir: PathBuf = directory.iter().take(len).collect();
                if !paths_ordered.contains(&partial_dir) {
                    paths_ordered.insert(partial_dir);
                }
            }
        }

        let mut paths_ordered: Vec<PathBuf> = paths_ordered.into_iter().collect();
        paths_ordered.sort_by_key(|path| path.iter().count());
        paths_ordered
    }
}

/// Fully traverse a PathType building a Vec<PathBuf>
fn traverse_paths_ordered(item: &PathType, current_path: Vec<OsString>, output: &mut Vec<PathBuf>) {
    match item {
        PathType::Directory(items) => {
            for (name, kind) in items.iter().sorted_by_key(|k| k.0) {
                let mut current_path = current_path.clone();
                current_path.push(name.to_owned());
                traverse_paths_ordered(kind, current_path, output);
            }
        }

        PathType::File => {
            let full_path: PathBuf = current_path.into_iter().collect();
            output.push(full_path);
        }
    }
}

/// The kind of a path inside the tree
#[derive(Debug, PartialEq, Eq)]
enum PathType {
    Directory(HashMap<OsString, PathType>),
    File,
}

impl PathType {
    /// The first `PathType` which contains the tree and everything
    fn root() -> Self {
        PathType::Directory(HashMap::default())
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, ffi::OsString, path::PathBuf};

    use crate::{tree::PathType, PathTree};

    /// Create a file with name
    macro_rules! f {
        ($name:tt) => {
            (OsString::from($name), PathType::File)
        };
    }

    /// Build an hashmap
    macro_rules! hm {
        ($($inner:tt)*) => {
            HashMap::from([$($inner)*])
        };
    }

    /// Create a dir with content
    macro_rules! d {
        ($name:tt, $($inner:tt)*) => {
            (OsString::from($name), PathType::Directory(hm!($($inner)*)))
        };
    }

    /// Create the root structure
    macro_rules! root {
        ($($inner:tt)*) => {
            PathType::Directory(hm![$($inner)*])
        };
    }

    #[test]
    fn test_path_tree_single_in_root() {
        let files = ["hehe.txt"];
        let tree: PathTree = files.iter().map(PathBuf::from).collect();

        let expected_structure = root!(f!("hehe.txt"));

        assert_eq!(tree.0, expected_structure);
    }

    #[test]
    fn test_path_tree_many_in_root() {
        let files = [
            "hehe.txt",
            "ciaociao.txt",
            "rusty-dick.dat",
            "no-extension-lol",
        ];
        let tree: PathTree = files.iter().map(PathBuf::from).collect();

        let expected_structure =
            PathType::Directory(files.iter().map(|path| f!(path)).collect::<HashMap<_, _>>());

        assert_eq!(tree.0, expected_structure);

        let directories = tree.directory_structure();
        let expected_dirs: Vec<PathBuf> = vec![];

        assert_eq!(directories.len(), expected_dirs.len());

        for dir in directories {
            assert!(expected_dirs.contains(&dir))
        }
    }

    #[test]
    fn test_path_tree_single_nested() {
        let files = ["some/path/lol/midget-porn.mp4"];
        let tree: PathTree = files.iter().map(PathBuf::from).collect();

        let expected_structure = root!(d!("some", d!("path", d!("lol", f!("midget-porn.mp4")))));

        assert_eq!(tree.0, expected_structure);

        let directories = tree.directory_structure();
        let expected_dirs: Vec<PathBuf> = vec!["some", "some/path", "some/path/lol"]
            .into_iter()
            .map(PathBuf::from)
            .collect();

        assert_eq!(directories.len(), expected_dirs.len());

        for dir in directories {
            assert!(expected_dirs.contains(&dir))
        }
    }

    #[test]
    fn test_path_tree_many_nested() {
        let files = ["some/path/lol/midget-porn.mp4", "some/path/lol.dat"];
        let tree: PathTree = files.iter().map(PathBuf::from).collect();

        let expected_structure = root!(d!(
            "some",
            d!("path", d!("lol", f!("midget-porn.mp4")), f!("lol.dat"))
        ));

        assert_eq!(tree.0, expected_structure);

        let directories = tree.directory_structure();
        let expected_dirs: Vec<PathBuf> = vec!["some", "some/path", "some/path/lol"]
            .into_iter()
            .map(PathBuf::from)
            .collect();

        assert_eq!(directories.len(), expected_dirs.len());

        for dir in directories {
            assert!(expected_dirs.contains(&dir))
        }
    }

    #[test]
    fn test_path_tree_many_misc() {
        let files = [
            "bdsm/hard-sex-orgasm.mp3",
            "some/path/lol/midget-porn.mp4",
            "some/path/lol.dat",
            "some/links.txt",
            "super/mega/ultra/nested/dir/x.dat",
            "super/mega/ultra/nested/porn.mp4",
        ];
        let tree: PathTree = files.iter().map(PathBuf::from).collect();

        let expected_structure = root!(
            d!("bdsm", f!("hard-sex-orgasm.mp3")),
            d!(
                "some",
                d!("path", d!("lol", f!("midget-porn.mp4")), f!("lol.dat")),
                f!("links.txt")
            ),
            d!(
                "super",
                d!(
                    "mega",
                    d!(
                        "ultra",
                        d!("nested", d!("dir", f!("x.dat")), f!("porn.mp4"))
                    )
                )
            )
        );

        assert_eq!(tree.0, expected_structure);

        let directories = tree.directory_structure();
        let expected_dirs: Vec<PathBuf> = vec![
            "bdsm",
            "some",
            "some/path",
            "some/path/lol",
            "super",
            "super/mega",
            "super/mega/ultra",
            "super/mega/ultra/nested",
            "super/mega/ultra/nested/dir",
        ]
        .into_iter()
        .map(PathBuf::from)
        .collect();

        assert_eq!(directories.len(), expected_dirs.len());

        for dir in directories {
            assert!(expected_dirs.contains(&dir))
        }
    }
}
