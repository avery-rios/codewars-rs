use std::{ffi::OsStr, io, os::unix::process::CommandExt, process};

use rustix::fd::{AsRawFd, BorrowedFd};

pub mod fs;

pub fn call_command_at<S, I, SA>(pwd: BorrowedFd, program: S, args: I) -> io::Result<()>
where
    S: AsRef<OsStr>,
    SA: AsRef<OsStr>,
    I: IntoIterator<Item = SA>,
{
    let pwd = pwd.as_raw_fd();
    let s = unsafe {
        process::Command::new(program)
            .args(args)
            .pre_exec(move || {
                rustix::process::fchdir(BorrowedFd::borrow_raw(pwd)).map_err(std::io::Error::from)
            })
            .status()?
    };
    if let Some(c) = s.code() {
        println!("command exited with code {}", c);
    } else if !s.success() {
        println!("command exited with unknown error");
    }
    Ok(())
}
