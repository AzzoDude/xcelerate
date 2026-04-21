use std::process::Command;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::error::{XcelerateResult, XcelerateError};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(unix)]
use std::os::unix::process::CommandExt;

/// Global registry of active browser PIDs for emergency cleanup.
pub static REGISTERED_PIDS: Lazy<Mutex<Vec<u32>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Spawns a detached process that can outlive the parent.
/// 
/// Returns the PID of the spawned process.
pub fn spawn_detached(mut cmd: Command) -> XcelerateResult<u32> {
    #[cfg(windows)]
    {
        // CREATE_NEW_PROCESS_GROUP = 0x00000200
        // DETACHED_PROCESS = 0x00000008
        cmd.creation_flags(0x00000200 | 0x00000008);
    }

    #[cfg(unix)]
    {
        unsafe {
            cmd.pre_exec(|| {
                // Create a new session to detach from the parent terminal
                if libc::setsid() == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }

    let child = cmd.spawn().map_err(|e| XcelerateError::NotFound(format!("Failed to spawn detached process: {}", e)))?;
    let pid = child.id();
    
    // Register the PID for global cleanup
    ProcessRegistry::register(pid);

    Ok(pid)
}

/// A guard that manages the lifecycle of a browser process.
/// When dropped, it kills the process unless it was detached and NOT meant to be killed.
pub struct ProcessGuard {
    pub pid: u32,
    pub auto_kill: bool,
}

impl Drop for ProcessGuard {
    fn drop(&mut self) {
        if self.auto_kill {
            kill_pid(self.pid);
        }
        
        // Remove from global registry as it's already handled
        ProcessRegistry::unregister(self.pid);
    }
}

/// Forcefully kills a process by PID.
pub fn kill_pid(pid: u32) {
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/F", "/PID", &pid.to_string()])
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

/// Global cleanup function to be called on application exit if desired.
pub fn cleanup_all() {
    if let Ok(mut pids) = REGISTERED_PIDS.lock() {
        for &pid in pids.iter() {
            kill_pid(pid);
        }
        pids.clear();
    }
}

pub struct ProcessRegistry;

impl ProcessRegistry {
    pub fn register(pid: u32) {
        if let Ok(mut pids) = REGISTERED_PIDS.lock() {
            pids.push(pid);
        }
    }

    pub fn unregister(pid: u32) {
        if let Ok(mut pids) = REGISTERED_PIDS.lock() {
            pids.retain(|&p| p != pid);
        }
    }

    pub fn cleanup() {
        cleanup_all();
    }
}
