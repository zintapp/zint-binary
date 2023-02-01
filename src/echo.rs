// extracted and modified from 'rpassword' crate
// https://github.com/conradkleinespel/rpassword/blob/master/src/lib.rs
// under Apache 2.0 license


use libc::{c_int, tcsetattr, termios, ECHO, ECHONL, TCSANOW};
use std::io;
use std::mem;

pub struct HiddenInput {
    fd: i32,
    term_orig: termios,
}

impl HiddenInput {
    pub fn new(fd: i32) -> io::Result<HiddenInput> {
        // Make two copies of the terminal settings. The first one will be modified
        // and the second one will act as a backup for when we want to set the
        // terminal back to its original state.
        let mut term = safe_tcgetattr(fd)?;
        let term_orig = safe_tcgetattr(fd)?;

        // Hide the user input.
        term.c_lflag &= !ECHO;

        // also hide the NL character when the user hits ENTER
        // (because the 'User' will actually be automated input from Zint).
        term.c_lflag &= !ECHONL;

        // Save the settings for now.
        io_result(unsafe { tcsetattr(fd, TCSANOW, &term) })?;

        Ok(HiddenInput { fd, term_orig })
    }
}

impl Drop for HiddenInput {
    fn drop(&mut self) {
        // Set the the mode back to normal
        unsafe {
            tcsetattr(self.fd, TCSANOW, &self.term_orig);
        }
    }
}

/// Turns a C function return into an IO Result
fn io_result(ret: c_int) -> std::io::Result<()> {
    match ret {
        0 => Ok(()),
        _ => Err(std::io::Error::last_os_error()),
    }
}

fn safe_tcgetattr(fd: c_int) -> std::io::Result<termios> {
    let mut term = mem::MaybeUninit::<termios>::uninit();
    io_result(unsafe { ::libc::tcgetattr(fd, term.as_mut_ptr()) })?;
    Ok(unsafe { term.assume_init() })
}