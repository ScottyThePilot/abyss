//! Utilities involving manipulation of the host system, or
//! relating to the [`mod@std::env`], [`mod@std::io`], or [`mod@std::fs`] modules.

use std::env;
use std::io;
use std::path::PathBuf;



/// Gets the current executable's dir.
pub fn get_current_exe_dir() -> io::Result<PathBuf> {
  if let Some(manifest_dir) = env::var_os("CARGO_MANIFEST_DIR") {
    return Ok(PathBuf::from(manifest_dir));
  };

  let mut current_exe = dunce::canonicalize(env::current_exe()?)?;

  if current_exe.pop() {
    return Ok(current_exe);
  };

  Err(io::Error::new(
    io::ErrorKind::Other,
    "failed to find the application root"
  ))
}

/// Set the current directory to the current executable's dir.
pub fn set_current_dir_to_exe_dir() -> io::Result<()> {
  get_current_exe_dir().and_then(env::set_current_dir)
}
