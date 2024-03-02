use std::{ffi::OsStr, io, path::Path, process};

pub fn call_command_in<S, I, SA>(pwd: impl AsRef<Path>, program: S, args: I) -> io::Result<()>
where
    S: AsRef<OsStr>,
    SA: AsRef<OsStr>,
    I: IntoIterator<Item = SA>,
{
    let s = process::Command::new(program)
        .args(args)
        .current_dir(pwd)
        .status()?;
    if let Some(c) = s.code() {
        println!("command exited with code {}", c);
    } else if !s.success() {
        println!("command exited with unknown error");
    }
    Ok(())
}
