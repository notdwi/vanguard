use std::path::Path;
use std::process::Command;

use serde::Serialize;

use crate::error::{AppError, Result};

use super::ca;

#[derive(Debug, Clone, Serialize)]
pub struct TrustStorePlan {
    pub platform: String,
    /// Human-readable description of exactly what the install button will run.
    pub steps: Vec<String>,
    pub requires_elevation: bool,
    pub manual_instructions: Vec<String>,
}

pub fn plan(cert: &Path) -> TrustStorePlan {
    let path = cert.to_string_lossy().into_owned();
    if cfg!(windows) {
        TrustStorePlan {
            platform: "windows".into(),
            steps: vec![format!("certutil -addstore -f Root \"{path}\"")],
            requires_elevation: true,
            manual_instructions: vec![
                "Open certmgr.msc".into(),
                "Go to Trusted Root Certification Authorities > Certificates".into(),
                format!("Right-click > All Tasks > Import, and select {path}"),
            ],
        }
    } else {
        let (dir, refresh) = linux_target();
        TrustStorePlan {
            platform: "linux".into(),
            steps: vec![
                format!("cp \"{path}\" {dir}/vanguard-ca.crt"),
                refresh.to_string(),
            ],
            requires_elevation: true,
            manual_instructions: vec![
                format!("sudo cp \"{path}\" {dir}/vanguard-ca.crt"),
                format!("sudo {refresh}"),
                "Firefox and Chrome keep their own stores; import there too if needed.".into(),
            ],
        }
    }
}

pub fn is_installed(cert: &Path) -> bool {
    if cfg!(windows) {
        windows_store_contains()
    } else {
        let (dir, _) = linux_target();
        Path::new(dir).join("vanguard-ca.crt").exists() || cert.with_extension("installed").exists()
    }
}

pub fn install(cert: &Path) -> Result<()> {
    if !cert.exists() {
        return Err(AppError::Certificate("certificate file is missing".into()));
    }
    if cfg!(windows) {
        run_elevated_windows("certutil", &["-addstore", "-f", "Root", &cert.to_string_lossy()])
    } else {
        let (dir, refresh) = linux_target();
        let script = format!(
            "install -m 644 '{}' '{dir}/vanguard-ca.crt' && {refresh}",
            cert.to_string_lossy()
        );
        run_elevated_linux(&script)
    }
}

pub fn uninstall(cert: &Path) -> Result<()> {
    if cfg!(windows) {
        run_elevated_windows("certutil", &["-delstore", "Root", ca::CA_COMMON_NAME])
    } else {
        let (dir, refresh) = linux_target();
        let script = format!("rm -f '{dir}/vanguard-ca.crt' && {refresh}");
        let _ = cert;
        run_elevated_linux(&script)
    }
}

fn linux_target() -> (&'static str, &'static str) {
    if Path::new("/etc/pki/ca-trust/source/anchors").is_dir() {
        ("/etc/pki/ca-trust/source/anchors", "update-ca-trust extract")
    } else if Path::new("/etc/ca-certificates/trust-source/anchors").is_dir() {
        ("/etc/ca-certificates/trust-source/anchors", "trust extract-compat")
    } else {
        ("/usr/local/share/ca-certificates", "update-ca-certificates")
    }
}

#[cfg(windows)]
fn windows_store_contains() -> bool {
    let out = command("certutil").args(["-store", "Root"]).output();
    match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).contains(ca::CA_COMMON_NAME),
        Err(_) => false,
    }
}

#[cfg(not(windows))]
fn windows_store_contains() -> bool {
    false
}

#[cfg(windows)]
fn run_elevated_windows(exe: &str, args: &[&str]) -> Result<()> {
    let arg_list = args
        .iter()
        .map(|a| format!("'{}'", a.replace('\'', "''")))
        .collect::<Vec<_>>()
        .join(",");
    let ps = format!(
        "$p = Start-Process -FilePath '{exe}' -ArgumentList {arg_list} -Verb RunAs -Wait -PassThru; exit $p.ExitCode"
    );
    let status = command("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps])
        .status()
        .map_err(|e| AppError::Certificate(format!("could not start the elevation prompt: {e}")))?;
    if status.success() {
        Ok(())
    } else {
        Err(AppError::Certificate(
            "the elevated command did not complete; it may have been cancelled".into(),
        ))
    }
}

#[cfg(not(windows))]
fn run_elevated_windows(_exe: &str, _args: &[&str]) -> Result<()> {
    Err(AppError::Certificate("not supported on this platform".into()))
}

#[cfg(not(windows))]
fn run_elevated_linux(script: &str) -> Result<()> {
    let runner = ["pkexec", "sudo"]
        .into_iter()
        .find(|c| which(c))
        .ok_or_else(|| AppError::Certificate("neither pkexec nor sudo is available".into()))?;
    let status = Command::new(runner)
        .args(["sh", "-c", script])
        .status()
        .map_err(|e| AppError::Certificate(format!("could not run the elevated command: {e}")))?;
    if status.success() {
        Ok(())
    } else {
        Err(AppError::Certificate(
            "the elevated command did not complete; it may have been cancelled".into(),
        ))
    }
}

#[cfg(windows)]
fn run_elevated_linux(_script: &str) -> Result<()> {
    Err(AppError::Certificate("not supported on this platform".into()))
}

#[cfg(not(windows))]
fn which(bin: &str) -> bool {
    Command::new("sh")
        .args(["-c", &format!("command -v {bin}")])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn command(exe: &str) -> Command {
    #[allow(unused_mut)]
    let mut cmd = Command::new(exe);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    cmd
}
