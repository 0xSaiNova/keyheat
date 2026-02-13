use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub fn pid_file_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("keyheat")
        .join("keyheat.pid")
}

pub fn read_pid() -> Option<u32> {
    let path = pid_file_path();
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

pub fn write_pid(pid: u32) -> Result<()> {
    use std::fs::OpenOptions;
    use std::io::Write;

    let path = pid_file_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("failed to create data directory")?;
    }

    // Use create_new to atomically fail if file already exists
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .context("PID file already exists - daemon may already be running")?;

    file.write_all(pid.to_string().as_bytes())
        .context("failed to write PID file")?;

    Ok(())
}

pub fn remove_pid_file() -> Result<()> {
    let path = pid_file_path();
    if path.exists() {
        fs::remove_file(&path).context("failed to remove PID file")?;
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn is_process_alive(pid: u32) -> bool {
    use std::process::Command;
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(target_os = "windows")]
pub fn is_process_alive(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle != 0 {
            CloseHandle(handle);
            true
        } else {
            false
        }
    }
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub fn is_process_alive(_pid: u32) -> bool {
    false
}

pub fn check_running() -> Option<u32> {
    let pid = read_pid()?;
    if is_process_alive(pid) {
        Some(pid)
    } else {
        let _ = remove_pid_file();
        None
    }
}

#[cfg(target_os = "linux")]
pub fn get_process_start_time(pid: u32) -> Option<std::time::SystemTime> {
    use std::time::{Duration, UNIX_EPOCH};

    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let parts: Vec<&str> = stat.split_whitespace().collect();
    let starttime_ticks: u64 = parts.get(21)?.parse().ok()?;

    let clock_ticks_per_sec: u64 = unsafe { libc::sysconf(libc::_SC_CLK_TCK) as u64 };
    let boot_time = get_boot_time()?;

    let start_secs = boot_time + (starttime_ticks / clock_ticks_per_sec);
    Some(UNIX_EPOCH + Duration::from_secs(start_secs))
}

#[cfg(target_os = "linux")]
fn get_boot_time() -> Option<u64> {
    let stat = fs::read_to_string("/proc/stat").ok()?;
    for line in stat.lines() {
        if let Some(rest) = line.strip_prefix("btime ") {
            return rest.trim().parse().ok();
        }
    }
    None
}

#[cfg(target_os = "windows")]
pub fn get_process_start_time(pid: u32) -> Option<std::time::SystemTime> {
    use std::time::{Duration, UNIX_EPOCH};
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{
        GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle == 0 {
            return None;
        }

        let mut creation: i64 = 0;
        let mut exit: i64 = 0;
        let mut kernel: i64 = 0;
        let mut user: i64 = 0;

        let result = GetProcessTimes(
            handle,
            &mut creation as *mut i64 as *mut _,
            &mut exit as *mut i64 as *mut _,
            &mut kernel as *mut i64 as *mut _,
            &mut user as *mut i64 as *mut _,
        );

        CloseHandle(handle);

        if result == 0 {
            return None;
        }

        // FILETIME is 100ns intervals since Jan 1, 1601
        // Convert to Unix epoch (Jan 1, 1970)
        const FILETIME_UNIX_DIFF: i64 = 116444736000000000;
        let unix_100ns = creation - FILETIME_UNIX_DIFF;
        if unix_100ns < 0 {
            return None;
        }

        let secs = (unix_100ns / 10_000_000) as u64;
        let nanos = ((unix_100ns % 10_000_000) * 100) as u32;
        Some(UNIX_EPOCH + Duration::new(secs, nanos))
    }
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub fn get_process_start_time(_pid: u32) -> Option<std::time::SystemTime> {
    None
}

pub fn format_uptime(start: std::time::SystemTime) -> String {
    let elapsed = start.elapsed().unwrap_or_default();
    let secs = elapsed.as_secs();

    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;

    if hours > 0 {
        format!("{hours}h {mins}m")
    } else {
        format!("{mins}m")
    }
}

#[cfg(target_os = "linux")]
pub fn kill_process(pid: u32) -> Result<()> {
    use std::process::Command;
    let status = Command::new("kill")
        .arg(pid.to_string())
        .status()
        .context("failed to send kill signal")?;

    if !status.success() {
        anyhow::bail!("kill command failed");
    }
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn kill_process(pid: u32) -> Result<()> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if handle == 0 {
            anyhow::bail!("failed to open process");
        }

        let result = TerminateProcess(handle, 0);
        CloseHandle(handle);

        if result == 0 {
            anyhow::bail!("failed to terminate process");
        }
    }
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub fn kill_process(_pid: u32) -> Result<()> {
    anyhow::bail!("kill not supported on this platform")
}

#[cfg(target_os = "linux")]
pub fn spawn_daemon() -> Result<u32> {
    use std::os::unix::process::CommandExt;
    use std::process::{Command, Stdio};

    let exe = std::env::current_exe().context("failed to get executable path")?;

    unsafe {
        let child = Command::new(&exe)
            .arg("run")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .pre_exec(|| {
                libc::setsid();
                Ok(())
            })
            .spawn()
            .context("failed to spawn daemon")?;

        Ok(child.id())
    }
}

#[cfg(target_os = "windows")]
pub fn spawn_daemon() -> Result<u32> {
    use std::os::windows::process::CommandExt;
    use std::process::{Command, Stdio};

    const DETACHED_PROCESS: u32 = 0x00000008;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let exe = std::env::current_exe().context("failed to get executable path")?;

    let child = Command::new(&exe)
        .arg("run")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW)
        .spawn()
        .context("failed to spawn daemon")?;

    Ok(child.id())
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub fn spawn_daemon() -> Result<u32> {
    anyhow::bail!("daemon mode not supported on this platform")
}
