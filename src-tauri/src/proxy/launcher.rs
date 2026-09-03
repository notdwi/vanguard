use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::{AppError, Result};
use crate::tls::install::command;

use super::firefox;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BrowserKind {
    Chromium,
    Firefox,
}

#[derive(Debug, Clone, Serialize)]
pub struct BrowserOption {
    pub id: String,
    pub name: String,
    pub path: String,
    pub kind: BrowserKind,
    /// True when the browser reads the operating system trust store, so
    /// Install CA is enough. Firefox on Linux needs its own import.
    pub uses_system_trust: bool,
}

/// Browsers we can point at the proxy using an isolated profile. The system
/// proxy settings are never touched.
pub fn available() -> Vec<BrowserOption> {
    let mut seen: Vec<String> = Vec::new();
    let mut out = Vec::new();

    for (id, name, kind, path) in candidates() {
        if !Path::new(&path).exists() || seen.contains(&id.to_string()) {
            continue;
        }
        seen.push(id.to_string());
        out.push(BrowserOption {
            id: id.to_string(),
            name: name.to_string(),
            path,
            kind,
            uses_system_trust: kind == BrowserKind::Chromium
                || cfg!(any(windows, target_os = "macos")),
        });
    }
    out
}

pub fn launch(
    browser_id: &str,
    proxy_addr: &str,
    profile_root: &Path,
    ca_cert: &Path,
    url: &str,
) -> Result<()> {
    let option = available()
        .into_iter()
        .find(|b| b.id == browser_id)
        .ok_or_else(|| AppError::Invalid(format!("browser {browser_id} was not found")))?;

    let profile = profile_dir(profile_root, browser_id);
    std::fs::create_dir_all(&profile)?;

    match option.kind {
        BrowserKind::Chromium => launch_chromium(&option, proxy_addr, &profile, url),
        BrowserKind::Firefox => launch_firefox(&option, proxy_addr, &profile, ca_cert, url),
    }
}

fn launch_chromium(
    option: &BrowserOption,
    proxy_addr: &str,
    profile: &Path,
    url: &str,
) -> Result<()> {
    let mut cmd = command(&option.path);
    cmd.arg(format!("--proxy-server=http://{proxy_addr}"));
    cmd.arg(format!("--user-data-dir={}", profile.to_string_lossy()));
    cmd.arg("--proxy-bypass-list=<-loopback>");
    cmd.arg("--no-first-run");
    cmd.arg("--no-default-browser-check");
    cmd.arg("--disable-features=ChromeWhatsNewUI");
    if !url.is_empty() {
        cmd.arg(url);
    }
    cmd.spawn()
        .map_err(|e| AppError::Proxy(format!("could not start {}: {e}", option.name)))?;
    Ok(())
}

fn launch_firefox(
    option: &BrowserOption,
    proxy_addr: &str,
    profile: &Path,
    ca_cert: &Path,
    url: &str,
) -> Result<()> {
    let (host, port) = split_addr(proxy_addr)?;
    firefox::prepare_profile(profile, &host, port)?;
    if let Err(e) = firefox::trust_ca(profile, ca_cert) {
        tracing::warn!("could not add the CA to the Firefox profile: {e}");
    }
    firefox::spawn(&option.path, profile, url)
}

fn split_addr(addr: &str) -> Result<(String, u16)> {
    let (host, port) = addr
        .rsplit_once(':')
        .ok_or_else(|| AppError::Invalid(format!("malformed proxy address {addr}")))?;
    let port: u16 = port
        .parse()
        .map_err(|_| AppError::Invalid(format!("malformed proxy port in {addr}")))?;
    Ok((host.trim_matches(['[', ']']).to_string(), port))
}

pub fn profile_dir(root: &Path, browser_id: &str) -> PathBuf {
    root.join("browser-profiles").join(browser_id)
}

pub fn clear_profile(root: &Path, browser_id: &str) -> Result<()> {
    let dir = profile_dir(root, browser_id);
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    Ok(())
}

#[cfg(windows)]
fn candidates() -> Vec<(&'static str, &'static str, BrowserKind, String)> {
    let pf = std::env::var("ProgramFiles").unwrap_or_default();
    let pf86 = std::env::var("ProgramFiles(x86)").unwrap_or_default();
    let local = std::env::var("LOCALAPPDATA").unwrap_or_default();
    vec![
        ("firefox", "Mozilla Firefox", BrowserKind::Firefox, format!("{pf}\\Mozilla Firefox\\firefox.exe")),
        ("firefox", "Mozilla Firefox", BrowserKind::Firefox, format!("{pf86}\\Mozilla Firefox\\firefox.exe")),
        // Microsoft Store builds are reached through their execution alias.
        ("firefox", "Mozilla Firefox", BrowserKind::Firefox, format!("{local}\\Microsoft\\WindowsApps\\firefox.exe")),
        ("firefox-dev", "Firefox Developer Edition", BrowserKind::Firefox, format!("{pf}\\Firefox Developer Edition\\firefox.exe")),
        ("chrome", "Google Chrome", BrowserKind::Chromium, format!("{pf}\\Google\\Chrome\\Application\\chrome.exe")),
        ("chrome", "Google Chrome", BrowserKind::Chromium, format!("{pf86}\\Google\\Chrome\\Application\\chrome.exe")),
        ("edge", "Microsoft Edge", BrowserKind::Chromium, format!("{pf86}\\Microsoft\\Edge\\Application\\msedge.exe")),
        ("edge", "Microsoft Edge", BrowserKind::Chromium, format!("{pf}\\Microsoft\\Edge\\Application\\msedge.exe")),
        ("brave", "Brave", BrowserKind::Chromium, format!("{pf}\\BraveSoftware\\Brave-Browser\\Application\\brave.exe")),
        ("chromium", "Chromium", BrowserKind::Chromium, format!("{local}\\Chromium\\Application\\chrome.exe")),
    ]
}

#[cfg(not(windows))]
fn candidates() -> Vec<(&'static str, &'static str, BrowserKind, String)> {
    let entries: &[(&'static str, &'static str, BrowserKind, &[&str])] = &[
        (
            "firefox",
            "Mozilla Firefox",
            BrowserKind::Firefox,
            &["/usr/bin/firefox", "/usr/lib/firefox/firefox", "/snap/bin/firefox", "/opt/firefox/firefox", "/Applications/Firefox.app/Contents/MacOS/firefox"],
        ),
        (
            "chrome",
            "Google Chrome",
            BrowserKind::Chromium,
            &["/usr/bin/google-chrome", "/usr/bin/google-chrome-stable", "/opt/google/chrome/chrome", "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"],
        ),
        (
            "chromium",
            "Chromium",
            BrowserKind::Chromium,
            &["/usr/bin/chromium", "/usr/bin/chromium-browser", "/snap/bin/chromium"],
        ),
        ("brave", "Brave", BrowserKind::Chromium, &["/usr/bin/brave-browser", "/usr/bin/brave"]),
        (
            "edge",
            "Microsoft Edge",
            BrowserKind::Chromium,
            &["/usr/bin/microsoft-edge", "/usr/bin/microsoft-edge-stable"],
        ),
    ];

    entries
        .iter()
        .flat_map(|(id, label, kind, paths)| {
            paths.iter().map(move |p| (*id, *label, *kind, p.to_string()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_proxy_addresses() {
        assert_eq!(split_addr("127.0.0.1:8080").unwrap(), ("127.0.0.1".into(), 8080));
        assert!(split_addr("nonsense").is_err());
    }
}
