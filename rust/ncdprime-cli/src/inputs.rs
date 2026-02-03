use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct InputItem {
    pub label: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct InputSet {
    #[allow(dead_code)]
    pub name: String,
    pub items: Vec<InputItem>,
}

#[derive(Debug, Clone)]
pub enum SetSpec {
    Dir { path: PathBuf },
    File { path: PathBuf },
    List { path: PathBuf },
    Literal { text: String, label: Option<String> },
}

pub fn auto_detect_set_spec(arg: &str, list: bool) -> Result<SetSpec> {
    if list {
        return Ok(SetSpec::List {
            path: PathBuf::from(arg),
        });
    }

    let p = Path::new(arg);
    if let Ok(md) = fs::metadata(p) {
        if md.is_dir() {
            return Ok(SetSpec::Dir {
                path: p.to_path_buf(),
            });
        }
        if md.is_file() {
            return Ok(SetSpec::File {
                path: p.to_path_buf(),
            });
        }
    }

    Ok(SetSpec::Literal {
        text: arg.to_string(),
        label: None,
    })
}

pub fn load_set(spec: &SetSpec) -> Result<InputSet> {
    match spec {
        SetSpec::Dir { path } => {
            let mut entries: Vec<_> = fs::read_dir(path)
                .with_context(|| format!("read_dir({})", path.display()))?
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
                .collect();

            entries.sort_by_key(|e| e.file_name());

            let mut items = Vec::with_capacity(entries.len());
            for e in entries {
                let file_path = e.path();
                let bytes = fs::read(&file_path)
                    .with_context(|| format!("read({})", file_path.display()))?;
                let label = file_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| file_path.to_string_lossy().into_owned());
                items.push(InputItem { label, bytes });
            }

            Ok(InputSet {
                name: path.display().to_string(),
                items,
            })
        }

        SetSpec::File { path } => {
            let bytes = fs::read(path).with_context(|| format!("read({})", path.display()))?;
            let label = path
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| path.to_string_lossy().into_owned());
            Ok(InputSet {
                name: path.display().to_string(),
                items: vec![InputItem { label, bytes }],
            })
        }

        SetSpec::List { path } => {
            let raw = fs::read_to_string(path)
                .with_context(|| format!("read_to_string({})", path.display()))?;

            let paths: Vec<&str> = raw
                .lines()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty() && !s.starts_with('#'))
                .collect();

            let mut items = Vec::with_capacity(paths.len());
            for p in paths {
                let pb = PathBuf::from(p);
                if !pb.exists() {
                    return Err(anyhow!("listed path does not exist: {p}"));
                }
                let bytes = fs::read(&pb).with_context(|| format!("read({p})"))?;
                let label = pb
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or(p)
                    .to_string();
                items.push(InputItem { label, bytes });
            }

            Ok(InputSet {
                name: path.display().to_string(),
                items,
            })
        }

        SetSpec::Literal { text, label } => {
            let label = label.clone().unwrap_or_else(|| "literal".to_string());
            Ok(InputSet {
                name: label.clone(),
                items: vec![InputItem {
                    label,
                    bytes: text.as_bytes().to_vec(),
                }],
            })
        }
    }
}
