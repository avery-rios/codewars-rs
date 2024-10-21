use std::{ffi::CStr, mem::MaybeUninit, path::Path};

use rustix::{
    fd::{AsFd, BorrowedFd, OwnedFd},
    fs::{self, AtFlags, FileType, Mode, OFlags},
    io::Errno,
    path::Arg,
};

pub fn open_dirfd(root: impl Arg) -> Result<OwnedFd, Errno> {
    fs::open(
        root,
        OFlags::PATH | OFlags::DIRECTORY | OFlags::CLOEXEC,
        Mode::empty(),
    )
}
pub fn mkdirat(root: BorrowedFd, path: impl Arg) -> Result<(), Errno> {
    fs::mkdirat(root, path, Mode::from_raw_mode(0o777))
}

pub fn mkdir_all_at(root: BorrowedFd, path: &Path) -> Result<(), std::io::Error> {
    if path == Path::new("") {
        return Ok(());
    }
    match mkdirat(root, path) {
        Ok(()) => Ok(()),
        Err(Errno::NOENT) => match path.parent() {
            Some(p) => {
                mkdir_all_at(root, p)?;
                mkdirat(root, path)?;
                Ok(())
            }
            None => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "failed to create all dir",
            )),
        },
        Err(Errno::EXIST) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub fn write(root: BorrowedFd, path: impl Arg, data: impl AsRef<[u8]>) -> Result<(), Errno> {
    let file = fs::openat(
        root,
        path,
        OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::CLOEXEC,
        Mode::from_raw_mode(0o666),
    )?;
    let data = data.as_ref();
    let mut off = 0;
    while off < data.len() {
        off += rustix::io::write(file.as_fd(), &data[off..])?;
    }
    Ok(())
}
pub fn read(root: BorrowedFd, path: impl Arg) -> Result<Vec<u8>, Errno> {
    let mut ret = Vec::new();
    let file = fs::openat(root, path, OFlags::RDONLY | OFlags::CLOEXEC, Mode::empty())?;
    let mut buf = [MaybeUninit::uninit(); 8192];
    loop {
        let (buf, _) = rustix::io::read_uninit(file.as_fd(), &mut buf)?;
        if buf.is_empty() {
            break;
        } else {
            ret.extend_from_slice(buf);
        }
    }
    Ok(ret)
}
pub fn read_to_string(root: BorrowedFd, path: impl Arg) -> Result<String, std::io::Error> {
    String::from_utf8(read(root, path)?)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

pub fn remove_at(root: BorrowedFd, path: impl Arg) -> Result<(), Errno> {
    // unlinkat does not follow symlink
    match fs::unlinkat(root, path, AtFlags::empty()) {
        Ok(()) => Ok(()),
        Err(Errno::NOENT) => Ok(()),
        Err(e) => Err(e),
    }
}

// code adapted from cap-primitives 3.3.0
pub fn remove_dir_all_at(root: BorrowedFd, path: &CStr) -> Result<(), Errno> {
    fn remove_dir_rec(root: BorrowedFd, path: &CStr) -> Result<(), Errno> {
        let dir = fs::openat(root, path, OFlags::RDONLY | OFlags::CLOEXEC, Mode::empty())?;
        for child in fs::Dir::read_from(dir.as_fd())? {
            match child.and_then(|ch| {
                if ch.file_name() == c"." || ch.file_name() == c".." {
                    Ok(())
                } else {
                    match ch.file_type() {
                        FileType::Directory => remove_dir_rec(dir.as_fd(), ch.file_name()),
                        _ => fs::unlinkat(dir.as_fd(), ch.file_name(), AtFlags::empty()),
                    }
                }
            }) {
                Ok(()) => (),
                // ignore not exist error to avoid race condition
                Err(Errno::NOENT) => (),
                Err(e) => return Err(e),
            }
        }
        match fs::unlinkat(root, path, AtFlags::REMOVEDIR) {
            Ok(()) => Ok(()),
            Err(Errno::NOENT) => Ok(()),
            Err(e) => Err(e),
        }
    }

    match fs::statat(root, path, AtFlags::SYMLINK_NOFOLLOW) {
        Ok(s) => match FileType::from_raw_mode(s.st_mode) {
            FileType::Symlink => remove_at(root, path),
            _ => remove_dir_rec(root, path),
        },
        Err(Errno::NOENT) => Ok(()),
        Err(e) => Err(e),
    }
}
