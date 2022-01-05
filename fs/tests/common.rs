use std::{fs::File, io::Write, path::Path};

pub fn generate_files(path: impl AsRef<Path>, files_count: usize, file_length: usize) {
    for i in 0..files_count {
        let mut filename = path.as_ref().to_path_buf();
        filename.push(format!("file_{}", i));

        let mut out_file = File::create(filename).unwrap();

        let random_bytes: Vec<u8> = (0..file_length).map(|_| rand::random::<u8>()).collect();
        out_file.write_all(&random_bytes).unwrap();
    }
}
