use std::{
    collections::HashMap,
    ffi::OsString,
    path::{Path, PathBuf},
};

use itertools::Itertools;

/// Where the actual paths are stored
#[derive(Debug)]
pub struct PathTree(PathKind);

impl Default for PathTree {
    fn default() -> Self {
        Self(PathKind::root())
    }
}

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
            PathKind::Directory(contents) => contents,
            PathKind::File => panic!(
                "unexpected error: root is of type PathKind::File instead of PathKind::Directory"
            ),
        };
        let path_len = file_path.as_ref().iter().count();

        for (i, piece) in file_path.as_ref().iter().enumerate() {
            let piece = piece.to_owned();

            if current_path.get(&piece).is_some() {
                // path already exists, just traverse
                current_path = match current_path.get_mut(&piece).unwrap() {
                    PathKind::Directory (contents) => contents,
                    PathKind::File => panic!(
                        "unexpected error: {piece:?} is of type PathKind::File instead of PathKind::Directory"
                    ),
                };

                continue;
            }

            if i + 1 == path_len {
                // Current piece is a file
                current_path.insert(piece.to_owned(), PathKind::File);
            } else {
                // Current piece is a directory
                current_path.insert(piece.to_owned(), PathKind::Directory(HashMap::default()));

                current_path = match current_path.get_mut(&piece).unwrap() {
                    PathKind::Directory (contents ) => contents,
                    PathKind::File => panic!(
                        "unexpected error: {piece:?} is of type PathKind::File instead of PathKind::Directory"
                    ),
                };
            }
        }
    }

    pub fn print_ordered(&self) {
        let mut output = vec![];
        traverse_paths_ordered(&self.0, vec![], &mut output);

        let mut prev_dir = OsString::new();

        for path in output {
            let full_path_len = path.iter().count();
            let containing_dir: OsString = path.iter().take(full_path_len - 1).collect();
            let containing_dir_len = containing_dir.len();

            let whitespaces: String = (0..containing_dir_len).into_iter().map(|_| ' ').collect();
            let filename = path.iter().last().unwrap().to_string_lossy().to_string();

            if containing_dir != prev_dir {
                println!("├── {}", containing_dir.to_string_lossy());
            }

            println!("{whitespaces}├── {filename}");

            prev_dir = containing_dir;
        }
    }
}

/// Fully traverse a PathKind building a Vec<PathBuf>
fn traverse_paths_ordered(item: &PathKind, current_path: Vec<OsString>, output: &mut Vec<PathBuf>) {
    match item {
        PathKind::Directory(items) => {
            for (name, kind) in items.iter().sorted_by_key(|k| k.0) {
                let mut current_path = current_path.clone();
                current_path.push(name.to_owned());
                traverse_paths_ordered(kind, current_path, output);
            }
        }

        PathKind::File => {
            let full_path: PathBuf = current_path.into_iter().collect();
            output.push(full_path);
        }
    }
}

pub struct OrderedTree {
    paths: Vec<String>,
}

impl Iterator for OrderedTree {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.paths.pop()
    }
}

/// The kind of a path inside the tree
#[derive(Debug, PartialEq, Eq)]
enum PathKind {
    Directory(HashMap<OsString, PathKind>),
    File,
}

impl PathKind {
    /// The first `PathKind` which contains the tree and everything
    fn root() -> Self {
        PathKind::Directory(HashMap::default())
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, ffi::OsString, path::PathBuf};

    use crate::{tree::PathKind, PathTree};

    /// Create a file with name
    macro_rules! f {
        ($name:tt) => {
            (OsString::from($name), PathKind::File)
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
            (OsString::from($name), PathKind::Directory(hm!($($inner)*)))
        };
    }

    /// Create the root structure
    macro_rules! root {
        ($($inner:tt)*) => {
            PathKind::Directory(hm![$($inner)*])
        };
    }

    #[test]
    fn test_path_tree_single_in_root() {
        let files = ["hehe.txt"];
        let tree: PathTree = files.iter().map(PathBuf::from).collect();

        let expected = root!(f!("hehe.txt"));

        assert_eq!(tree.0, expected);
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

        let expected =
            PathKind::Directory(files.iter().map(|path| f!(path)).collect::<HashMap<_, _>>());

        assert_eq!(tree.0, expected);
    }

    #[test]
    fn test_path_tree_single_nested() {
        let files = ["some/path/lol/midget-porn.mp4"];
        let tree: PathTree = files.iter().map(PathBuf::from).collect();

        let expected = root!(d!("some", d!("path", d!("lol", f!("midget-porn.mp4")))));

        assert_eq!(tree.0, expected);
    }

    #[test]
    fn test_path_tree_many_nested() {
        let files = ["some/path/lol/midget-porn.mp4", "some/path/lol.dat"];
        let tree: PathTree = files.iter().map(PathBuf::from).collect();

        let expected = root!(d!(
            "some",
            d!("path", d!("lol", f!("midget-porn.mp4")), f!("lol.dat"))
        ));

        assert_eq!(tree.0, expected);
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

        let expected = root!(
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

        assert_eq!(tree.0, expected);
    }
}
