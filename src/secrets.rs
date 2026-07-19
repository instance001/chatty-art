use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum SecretEntry {
    Plain(String),
    Protected {
        protected_b64: String,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SecretsFile {
    #[serde(default)]
    providers: HashMap<String, SecretEntry>,
}

pub fn default_secrets_path(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join("config").join("cloud_secrets.json")
}

pub fn has_api_key(path: &Path, provider_id: &str) -> Result<bool> {
    Ok(read_api_key(path, provider_id)?.is_some())
}

pub fn read_api_key(path: &Path, provider_id: &str) -> Result<Option<String>> {
    let secrets = load_secrets(path)?;
    secrets
        .providers
        .get(provider_id.trim())
        .map(decode_secret_entry)
        .transpose()
}

pub fn save_api_key(path: &Path, provider_id: &str, api_key: &str) -> Result<()> {
    let provider_id = provider_id.trim();
    let api_key = api_key.trim();
    if provider_id.is_empty() {
        anyhow::bail!("provider id is required before saving an API key");
    }
    if api_key.is_empty() {
        anyhow::bail!("API key is empty");
    }
    let mut secrets = load_secrets(path)?;
    secrets
        .providers
        .insert(provider_id.to_string(), encode_secret_entry(api_key)?);
    save_secrets(path, &secrets)
}

pub fn delete_api_key(path: &Path, provider_id: &str) -> Result<()> {
    let mut secrets = load_secrets(path)?;
    secrets.providers.remove(provider_id.trim());
    save_secrets(path, &secrets)
}

fn load_secrets(path: &Path) -> Result<SecretsFile> {
    if !path.is_file() {
        return Ok(SecretsFile::default());
    }
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let secrets: SecretsFile =
        serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))?;
    Ok(secrets)
}

fn save_secrets(path: &Path, secrets: &SecretsFile) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create secrets dir {}", parent.display()))?;
    }
    let mut normalized = secrets.clone();
    for entry in normalized.providers.values_mut() {
        if let SecretEntry::Plain(value) = entry.clone() {
            *entry = encode_secret_entry(&value)?;
        }
    }
    let bytes = serde_json::to_vec_pretty(&normalized).context("serialize secrets file")?;
    std::fs::write(path, bytes).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn encode_secret_entry(api_key: &str) -> Result<SecretEntry> {
    #[cfg(windows)]
    {
        let protected = windows_dpapi::protect(api_key.as_bytes())?;
        return Ok(SecretEntry::Protected {
            protected_b64: BASE64_STANDARD.encode(protected),
        });
    }

    #[cfg(not(windows))]
    {
        Ok(SecretEntry::Plain(api_key.to_string()))
    }
}

fn decode_secret_entry(entry: &SecretEntry) -> Result<String> {
    match entry {
        SecretEntry::Plain(value) => Ok(value.clone()),
        SecretEntry::Protected { protected_b64 } => {
            let bytes = BASE64_STANDARD
                .decode(protected_b64)
                .context("decode protected API key")?;
            #[cfg(windows)]
            {
                let decrypted = windows_dpapi::unprotect(&bytes)?;
                String::from_utf8(decrypted).context("decode protected API key text")
            }
            #[cfg(not(windows))]
            {
                String::from_utf8(bytes).context("decode protected API key text")
            }
        }
    }
}

#[cfg(windows)]
mod windows_dpapi {
    use std::{ffi::c_void, ptr::null_mut, slice};

    use anyhow::{Context, Result, anyhow};

    #[repr(C)]
    struct DataBlob {
        cb_data: u32,
        pb_data: *mut u8,
    }

    #[link(name = "crypt32")]
    unsafe extern "system" {
        fn CryptProtectData(
            p_data_in: *mut DataBlob,
            sz_data_descr: *const u16,
            p_optional_entropy: *mut DataBlob,
            pv_reserved: *mut c_void,
            p_prompt_struct: *mut c_void,
            dw_flags: u32,
            p_data_out: *mut DataBlob,
        ) -> i32;

        fn CryptUnprotectData(
            p_data_in: *mut DataBlob,
            ppsz_data_descr: *mut *mut u16,
            p_optional_entropy: *mut DataBlob,
            pv_reserved: *mut c_void,
            p_prompt_struct: *mut c_void,
            dw_flags: u32,
            p_data_out: *mut DataBlob,
        ) -> i32;
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn LocalFree(h_mem: *mut c_void) -> *mut c_void;
    }

    pub fn protect(bytes: &[u8]) -> Result<Vec<u8>> {
        let mut input = DataBlob {
            cb_data: bytes.len() as u32,
            pb_data: bytes.as_ptr() as *mut u8,
        };
        let mut output = DataBlob {
            cb_data: 0,
            pb_data: null_mut(),
        };
        let success = unsafe {
            CryptProtectData(
                &mut input,
                std::ptr::null(),
                null_mut(),
                null_mut(),
                null_mut(),
                0,
                &mut output,
            )
        };
        if success == 0 {
            return Err(anyhow!(std::io::Error::last_os_error()))
                .context("protect API key with Windows DPAPI");
        }
        copy_and_free_blob(output)
    }

    pub fn unprotect(bytes: &[u8]) -> Result<Vec<u8>> {
        let mut input = DataBlob {
            cb_data: bytes.len() as u32,
            pb_data: bytes.as_ptr() as *mut u8,
        };
        let mut output = DataBlob {
            cb_data: 0,
            pb_data: null_mut(),
        };
        let success = unsafe {
            CryptUnprotectData(
                &mut input,
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
                0,
                &mut output,
            )
        };
        if success == 0 {
            return Err(anyhow!(std::io::Error::last_os_error()))
                .context("unprotect API key with Windows DPAPI");
        }
        copy_and_free_blob(output)
    }

    fn copy_and_free_blob(blob: DataBlob) -> Result<Vec<u8>> {
        if blob.pb_data.is_null() {
            return Ok(Vec::new());
        }
        let bytes = unsafe { slice::from_raw_parts(blob.pb_data, blob.cb_data as usize) }.to_vec();
        let freed = unsafe { LocalFree(blob.pb_data as *mut c_void) };
        if !freed.is_null() {
            return Err(anyhow!("LocalFree did not release DPAPI buffer cleanly"));
        }
        Ok(bytes)
    }
}
