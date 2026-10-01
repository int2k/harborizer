use std::{
    fs::{self, OpenOptions},
    io::{self, Read},
    path::Path,
};

pub fn read_limited_regular_file(path: &Path, max_bytes: u64) -> io::Result<String> {
    let path_metadata = fs::symlink_metadata(path)?;
    if path_metadata.file_type().is_symlink() || !path_metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected a regular file, not a symlink or special file",
        ));
    }

    let mut options = OpenOptions::new();
    options.read(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }

    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x0020_0000); // FILE_FLAG_OPEN_REPARSE_POINT
    }

    let file = options.open(path).map_err(|error| {
        #[cfg(unix)]
        if error.raw_os_error() == Some(libc::ELOOP) {
            return io::Error::new(io::ErrorKind::InvalidInput, "refusing to follow a symlink");
        }
        error
    })?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected a regular file, not a symlink or special file",
        ));
    }
    if metadata.len() > max_bytes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "file exceeds the configured size limit",
        ));
    }

    let mut bytes = Vec::new();
    file.take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max_bytes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "file exceeds the configured size limit",
        ));
    }
    String::from_utf8(bytes).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("file is not valid UTF-8: {error}"),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn reads_a_regular_file_within_the_limit() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("snapshot.json");
        fs::write(&path, "{\"valid\":true}").unwrap();
        assert_eq!(
            read_limited_regular_file(&path, 32).unwrap(),
            "{\"valid\":true}"
        );
    }

    #[test]
    fn rejects_a_file_larger_than_the_limit() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("snapshot.json");
        fs::write(&path, "0123456789").unwrap();
        assert_eq!(
            read_limited_regular_file(&path, 8).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }

    #[cfg(unix)]
    #[test]
    fn refuses_symlinks() {
        use std::os::unix::fs::symlink;
        let directory = tempfile::tempdir().unwrap();
        let target = directory.path().join("target");
        let link = directory.path().join("link");
        fs::write(&target, "secret").unwrap();
        symlink(&target, &link).unwrap();
        assert_eq!(
            read_limited_regular_file(&link, 32).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
    }

    #[test]
    fn rejects_directories_as_non_regular_files() {
        let directory = tempfile::tempdir().unwrap();
        assert_eq!(
            read_limited_regular_file(directory.path(), 32)
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidInput
        );
    }
}
