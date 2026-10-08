//! Browser process control: detached spawn, lifecycle guards, and exit cleanup.

use std::process::Command;
use std::sync::{LazyLock, Mutex};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use crate::error::{XcelerateError, XcelerateResult};

/// PIDs of browsers this process spawned, kept for cleanup on exit.
static REGISTERED_PIDS: LazyLock<Mutex<Vec<u32>>> = LazyLock::new(|| Mutex::new(Vec::new()));

/// Spawn `cmd` detached so the child outlives this process, and return its PID.
///
/// A detached browser is deliberately **not** registered for exit cleanup: the
/// `--detached` contract is that it outlives the command that spawned it.
pub fn spawn_detached(mut cmd: Command) -> XcelerateResult<u32> {
    #[cfg(windows)]
    {
        // CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS
        cmd.creation_flags(0x0000_0200 | 0x0000_0008);
    }

    #[cfg(unix)]
    {
        // A new session detaches the child from the parent terminal.
        unsafe {
            cmd.pre_exec(|| {
                if libc::setsid() == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }

    let child = cmd
        .spawn()
        .map_err(|e| XcelerateError::NotFound(format!("Failed to spawn detached process: {e}")))?;
    Ok(child.id())
}

/// Owns a browser process and kills it on drop unless it was spawned detached.
pub struct ProcessGuard {
    pub pid: u32,
    pub auto_kill: bool,
}

impl Drop for ProcessGuard {
    fn drop(&mut self) {
        if self.auto_kill {
            kill_pid(self.pid);
        }
        unregister(self.pid);
    }
}

/// Forcefully kills a process by PID, and on Windows its whole process tree.
///
/// A Chromium browser is a tree (browser + GPU + renderers); killing only the
/// root can leave the window orphaned, so `/T` is used deliberately.
pub fn kill_pid(pid: u32) {
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }

    #[cfg(unix)]
    {
        unsafe {
            libc::kill(pid as i32, libc::SIGTERM);
        }
    }
}

/// Kills every browser this process spawned and still tracks. Call on exit --
/// normal, error, or signal -- so a browser is never orphaned.
pub fn cleanup_all() {
    if let Ok(mut pids) = REGISTERED_PIDS.lock() {
        for &pid in pids.iter() {
            kill_pid(pid);
        }
        pids.clear();
    }
}

/// Tracks a spawned browser for [`cleanup_all`]. Used for browsers this process
/// owns (not detached, not `keep_alive`), so an abrupt exit does not orphan them.
pub(crate) fn register(pid: u32) {
    if let Ok(mut pids) = REGISTERED_PIDS.lock() {
        pids.push(pid);
    }
}

pub(crate) fn unregister(pid: u32) {
    if let Ok(mut pids) = REGISTERED_PIDS.lock() {
        pids.retain(|&p| p != pid);
    }
}
