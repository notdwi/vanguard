use std::path::{Path, PathBuf};

use rcgen::{
    BasicConstraints, CertificateParams, DistinguishedName, DnType, IsCa, Issuer, KeyPair,
    KeyUsagePurpose,
};
use serde::Serialize;

use crate::error::{AppError, Result};

pub const CA_COMMON_NAME: &str = "Vanguard Local CA";
const CERT_FILE: &str = "vanguard-ca.crt";
const KEY_FILE: &str = "vanguard-ca.key";
const VALID_YEARS: i64 = 5;

#[derive(Debug, Clone, Serialize)]
pub struct CaInfo {
    pub exists: bool,
    pub common_name: String,
    pub cert_path: String,
    pub key_path: String,
    pub fingerprint: Option<String>,
    pub not_after: Option<String>,
    pub installed: bool,
}

pub struct CaFiles {
    pub cert_pem: String,
    pub key_pem: String,
}

pub fn cert_path(root: &Path) -> PathBuf {
    root.join(CERT_FILE)
}

pub fn key_path(root: &Path) -> PathBuf {
    root.join(KEY_FILE)
}

pub fn exists(root: &Path) -> bool {
    cert_path(root).exists() && key_path(root).exists()
}

pub fn load(root: &Path) -> Result<CaFiles> {
    if !exists(root) {
        return Err(AppError::Certificate("no local CA has been generated yet".into()));
    }
    Ok(CaFiles {
        cert_pem: std::fs::read_to_string(cert_path(root))?,
        key_pem: std::fs::read_to_string(key_path(root))?,
    })
}

/// Creates a fresh self-signed CA. Never called implicitly: the user asks for
/// it from the CA screen, which explains what the certificate is for.
pub fn generate(root: &Path) -> Result<CaFiles> {
    std::fs::create_dir_all(root)?;

    let key_pair = KeyPair::generate()?;
    let mut params = CertificateParams::default();

    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, CA_COMMON_NAME);
    dn.push(DnType::OrganizationName, "Vanguard");
    params.distinguished_name = dn;

    params.is_ca = IsCa::Ca(BasicConstraints::Constrained(0));
    params.key_usages = vec![
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::CrlSign,
        KeyUsagePurpose::DigitalSignature,
    ];

    let now = time::OffsetDateTime::now_utc();
    params.not_before = now - time::Duration::days(1);
    params.not_after = now + time::Duration::days(365 * VALID_YEARS);

    let cert = params.self_signed(&key_pair)?;
    let files = CaFiles { cert_pem: cert.pem(), key_pem: key_pair.serialize_pem() };

    std::fs::write(cert_path(root), &files.cert_pem)?;
    std::fs::write(key_path(root), &files.key_pem)?;
    restrict_key_permissions(&key_path(root));

    Ok(files)
}

pub fn load_or_generate(root: &Path) -> Result<CaFiles> {
    if exists(root) {
        load(root)
    } else {
        generate(root)
    }
}

pub fn issuer(files: &CaFiles) -> Result<Issuer<'static, KeyPair>> {
    let key_pair = KeyPair::from_pem(&files.key_pem)?;
    Ok(Issuer::from_ca_cert_pem(&files.cert_pem, key_pair)?)
}

pub fn delete(root: &Path) -> Result<()> {
    let _ = std::fs::remove_file(cert_path(root));
    let _ = std::fs::remove_file(key_path(root));
    Ok(())
}

pub fn info(root: &Path, installed: bool) -> CaInfo {
    let exists = exists(root);
    let mut info = CaInfo {
        exists,
        common_name: CA_COMMON_NAME.to_string(),
        cert_path: cert_path(root).to_string_lossy().into_owned(),
        key_path: key_path(root).to_string_lossy().into_owned(),
        fingerprint: None,
        not_after: None,
        installed,
    };
    if exists {
        if let Ok(pem) = std::fs::read_to_string(cert_path(root)) {
            info.fingerprint = fingerprint(&pem);
        }
    }
    info
}

/// SHA-256 of the DER body, formatted the way certificate viewers show it.
pub fn fingerprint(pem: &str) -> Option<String> {
    let der = pem_to_der(pem)?;
    let digest = sha256(&der);
    Some(
        digest
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect::<Vec<_>>()
            .join(":"),
    )
}

pub fn pem_to_der(pem: &str) -> Option<Vec<u8>> {
    use base64::Engine;
    let body: String = pem
        .lines()
        .filter(|l| !l.starts_with("-----"))
        .collect::<Vec<_>>()
        .concat();
    base64::engine::general_purpose::STANDARD.decode(body.trim()).ok()
}

fn sha256(data: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

#[cfg(unix)]
fn restrict_key_permissions(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
}

#[cfg(not(unix))]
fn restrict_key_permissions(_path: &Path) {}
