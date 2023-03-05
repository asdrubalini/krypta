use std::{
    ffi::OsStr,
    path::Path,
    time::{Duration, UNIX_EPOCH},
};

use database::{models, traits::FetchAll, Database};
use fuse::{FileAttr, FileType, Filesystem, ReplyAttr, ReplyDirectory, ReplyEntry, Request};
use libc::ENOENT;

pub struct KryptaFS {
    db: Database,
}

impl Filesystem for KryptaFS {
    //fn lookup(&mut self, req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
    //let attr = FileAttr {
    //ino: 2,
    //size: 13,
    //blocks: 1,
    //atime: UNIX_EPOCH,
    //mtime: UNIX_EPOCH,
    //ctime: UNIX_EPOCH,
    //crtime: UNIX_EPOCH,
    //kind: FileType::RegularFile,
    //perm: 0o644,
    //nlink: 1,
    //uid: 501,
    //gid: 20,
    //rdev: 0,
    //flags: 0,
    //};

    //reply.entry(&Duration::from_secs(1), &attr, 0);
    //}

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        let root_dir: FileAttr = FileAttr {
            ino: 1,
            size: 0,
            blocks: 0,
            atime: UNIX_EPOCH,
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind: FileType::Directory,
            perm: 0o755,
            nlink: 2,
            uid: 501,
            gid: 20,
            rdev: 0,
            flags: 0,
        };

        reply.attr(&Duration::from_secs(1), &root_dir);

        //match ino {
        //1 => reply.attr(&Duration::from_secs(1), &root_dir),
        //_ => reply.error(ENOENT),
        //}
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        if ino != 1 {
            reply.error(ENOENT);
            return;
        }

        let files = models::File::fetch_all(&self.db).unwrap();

        let mut entries = vec![
            (1, FileType::Directory, ".".to_string()),
            (1, FileType::Directory, "..".to_string()),
            (2, FileType::RegularFile, "hello.txt".to_string()),
        ];

        for (i, file) in files.into_iter().enumerate() {
            entries.push((
                (i + 3) as u64,
                FileType::RegularFile,
                format!("{}", file.title),
            ));
        }

        for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
            // i + 1 means the index of the next entry
            reply.add(entry.0, (i + 1) as i64, entry.1, entry.2);
        }
        reply.ok();
    }
}

impl KryptaFS {
    pub fn mount(mountpoint: impl AsRef<Path>) {
        let options = ["-o", "ro", "-o", "local"]
            .iter()
            .map(|o| o.as_ref())
            .collect::<Vec<&OsStr>>();

        let db = database::connect_or_create().unwrap();
        fuse::mount(KryptaFS { db }, mountpoint, &options).unwrap();
    }
}
