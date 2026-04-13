use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
    path::Path,
};

#[derive(Debug, Clone)]
pub struct GgufSummary {
    architecture: Option<String>,
    tensor_names_lower: Vec<String>,
    tensor_shapes_lower: HashMap<String, Vec<u64>>,
}

impl GgufSummary {
    pub fn architecture(&self) -> Option<&str> {
        self.architecture.as_deref()
    }

    pub fn contains_tensor(&self, tensor_name: &str) -> bool {
        self.tensor_shapes_lower
            .contains_key(&tensor_name.to_ascii_lowercase())
    }

    pub fn contains_tensor_fragment(&self, fragment: &str) -> bool {
        let fragment = fragment.to_ascii_lowercase();
        self.tensor_names_lower
            .iter()
            .any(|name| name.contains(&fragment))
    }

    pub fn contains_any_tensor_fragment(&self, fragments: &[&str]) -> bool {
        fragments
            .iter()
            .any(|fragment| self.contains_tensor_fragment(fragment))
    }
}

pub fn inspect_gguf(path: &Path) -> Option<GgufSummary> {
    let mut file = File::open(path).ok()?;

    let mut magic = [0_u8; 4];
    file.read_exact(&mut magic).ok()?;
    if &magic != b"GGUF" {
        return None;
    }

    let version = read_u32(&mut file).ok()?;
    let tensor_count = if version >= 2 {
        read_u64(&mut file).ok()?
    } else {
        read_u32(&mut file).ok()? as u64
    };
    let metadata_count = if version >= 2 {
        read_u64(&mut file).ok()?
    } else {
        read_u32(&mut file).ok()? as u64
    };

    let mut architecture = None;
    for _ in 0..metadata_count {
        let key = read_string(&mut file).ok()?;
        let value_type = read_u32(&mut file).ok()?;

        match (key.as_str(), value_type) {
            ("general.architecture", GGUF_TYPE_STRING) => {
                architecture = Some(read_string(&mut file).ok()?.to_ascii_lowercase());
            }
            _ => skip_value(&mut file, value_type).ok()?,
        }
    }

    let mut tensor_names_lower = Vec::with_capacity(tensor_count.min(2048) as usize);
    let mut tensor_shapes_lower = HashMap::with_capacity(tensor_count.min(2048) as usize);

    for _ in 0..tensor_count {
        let name = read_string(&mut file).ok()?;
        let lower_name = name.to_ascii_lowercase();
        let n_dims = read_u32(&mut file).ok()? as usize;
        let mut dims = Vec::with_capacity(n_dims);
        for _ in 0..n_dims {
            dims.push(read_u64(&mut file).ok()?);
        }
        read_u32(&mut file).ok()?;
        read_u64(&mut file).ok()?;

        tensor_names_lower.push(lower_name.clone());
        tensor_shapes_lower.insert(lower_name, dims);
    }

    Some(GgufSummary {
        architecture,
        tensor_names_lower,
        tensor_shapes_lower,
    })
}

const GGUF_TYPE_STRING: u32 = 8;
const GGUF_TYPE_ARRAY: u32 = 9;

fn read_u32(reader: &mut File) -> io::Result<u32> {
    let mut buffer = [0_u8; 4];
    reader.read_exact(&mut buffer)?;
    Ok(u32::from_le_bytes(buffer))
}

fn read_u64(reader: &mut File) -> io::Result<u64> {
    let mut buffer = [0_u8; 8];
    reader.read_exact(&mut buffer)?;
    Ok(u64::from_le_bytes(buffer))
}

fn read_string(reader: &mut File) -> io::Result<String> {
    let length = read_u64(reader)? as usize;
    let mut buffer = vec![0_u8; length];
    reader.read_exact(&mut buffer)?;
    Ok(String::from_utf8_lossy(&buffer).into_owned())
}

fn skip_value(reader: &mut File, value_type: u32) -> io::Result<()> {
    match value_type {
        0 | 1 | 7 => skip_bytes(reader, 1),
        2 | 3 => skip_bytes(reader, 2),
        4 | 5 | 6 => skip_bytes(reader, 4),
        10 | 11 | 12 => skip_bytes(reader, 8),
        GGUF_TYPE_STRING => {
            let length = read_u64(reader)? as usize;
            skip_bytes(reader, length)
        }
        GGUF_TYPE_ARRAY => {
            let element_type = read_u32(reader)?;
            let length = read_u64(reader)?;
            for _ in 0..length {
                skip_value(reader, element_type)?;
            }
            Ok(())
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unsupported GGUF metadata value type {value_type}"),
        )),
    }
}

fn skip_bytes(reader: &mut File, length: usize) -> io::Result<()> {
    let mut buffer = vec![0_u8; length.min(8192)];
    let mut remaining = length;

    while remaining > 0 {
        let chunk = remaining.min(buffer.len());
        reader.read_exact(&mut buffer[..chunk])?;
        remaining -= chunk;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::inspect_gguf;

    fn temp_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("chatty-art-gguf-{label}-{unique}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn encode_string(value: &str) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
        bytes.extend_from_slice(value.as_bytes());
        bytes
    }

    #[test]
    fn reads_architecture_and_tensor_shapes() {
        let dir = temp_dir("summary");
        let path = dir.join("tiny.gguf");

        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"GGUF");
        bytes.extend_from_slice(&3_u32.to_le_bytes());
        bytes.extend_from_slice(&2_u64.to_le_bytes());
        bytes.extend_from_slice(&1_u64.to_le_bytes());

        bytes.extend_from_slice(&encode_string("general.architecture"));
        bytes.extend_from_slice(&8_u32.to_le_bytes());
        bytes.extend_from_slice(&encode_string("qwen3"));

        bytes.extend_from_slice(&encode_string("token_embd.weight"));
        bytes.extend_from_slice(&2_u32.to_le_bytes());
        bytes.extend_from_slice(&32000_u64.to_le_bytes());
        bytes.extend_from_slice(&4096_u64.to_le_bytes());
        bytes.extend_from_slice(&0_u32.to_le_bytes());
        bytes.extend_from_slice(&0_u64.to_le_bytes());

        bytes.extend_from_slice(&encode_string("patch_embedding.weight"));
        bytes.extend_from_slice(&5_u32.to_le_bytes());
        bytes.extend_from_slice(&2_u64.to_le_bytes());
        bytes.extend_from_slice(&2_u64.to_le_bytes());
        bytes.extend_from_slice(&1_u64.to_le_bytes());
        bytes.extend_from_slice(&48_u64.to_le_bytes());
        bytes.extend_from_slice(&5120_u64.to_le_bytes());
        bytes.extend_from_slice(&0_u32.to_le_bytes());
        bytes.extend_from_slice(&64_u64.to_le_bytes());

        fs::write(&path, bytes).unwrap();

        let summary = inspect_gguf(&path).unwrap();
        assert_eq!(summary.architecture(), Some("qwen3"));
        assert!(summary.contains_tensor("token_embd.weight"));
        assert!(summary.contains_tensor_fragment("patch_embedding"));
        assert_eq!(
            summary.tensor_shapes_lower["patch_embedding.weight"].len(),
            5
        );
    }
}
