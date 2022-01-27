use std::{
    collections::HashMap,
    ffi::OsString,
    path::{Path, PathBuf},
};

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

    pub fn ordered(self) -> OrderedTree {
        OrderedTree { _tree: self }
    }
}

pub struct OrderedTree {
    _tree: PathTree,
}

impl Iterator for OrderedTree {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
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

    macro_rules! f {
        ($name:tt) => {
            (OsString::from($name), PathKind::File)
        };
    }

    macro_rules! hm {
        ($($inner:tt)*) => {
            HashMap::from([$($inner)*])
        };
    }

    macro_rules! d {
        ($name:tt, $($inner:tt)*) => {
            (OsString::from($name), PathKind::Directory(hm!($($inner)*)))
        };
    }

    #[test]
    fn test_path_tree_single_in_root() {
        let files = ["hehe.txt"];
        let tree: PathTree = files.iter().map(PathBuf::from).collect();

        let expected = PathKind::Directory(hm![f!("hehe.txt")]);

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

        let expected = PathKind::Directory(hm![d!(
            "some",
            d!("path", d!("lol", f!("midget-porn.mp4")))
        )]);

        assert_eq!(tree.0, expected);
    }

    #[test]
    fn test_path_tree_many_nested() {
        let files = ["some/path/lol/midget-porn.mp4", "some/path/lol.dat"];
        let tree: PathTree = files.iter().map(PathBuf::from).collect();

        let expected = PathKind::Directory(hm!(d!(
            "some",
            d!("path", d!("lol", f!("midget-porn.mp4")), f!("lol.dat"))
        )));

        assert_eq!(tree.0, expected);
    }
}
