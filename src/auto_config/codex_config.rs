use chrono::{FixedOffset, Utc};
use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::{Array, DocumentMut, Item, Table, Value};

const RECOMMENDED_STATUS_LINE: &[&str] = &[
    "model-with-reasoning",
    "context-remaining",
    "current-dir",
    "git-branch",
];

/// 自动配置 Codex 的原生 TUI footer。
pub struct CodexConfigurator;

impl CodexConfigurator {
    /// 获取 Codex 用户配置路径。
    pub fn get_config_path() -> Option<PathBuf> {
        dirs::home_dir().map(|home| home.join(".codex").join("config.toml"))
    }

    /// 初始化默认 Codex 配置。
    pub fn configure_statusline() -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::get_config_path().ok_or("无法找到 Codex 配置路径")?;
        Self::configure_statusline_at(&path)
    }

    /// 在指定路径初始化 Codex 配置，供测试和诊断使用。
    pub fn configure_statusline_at(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let existed = path.exists();
        let mut document = if existed {
            let content = fs::read_to_string(path)?;
            content.parse::<DocumentMut>()?
        } else {
            DocumentMut::new()
        };

        if let Some(tui) = document.get("tui") {
            if !tui.is_table() {
                return Err("Codex 配置中的 tui 必须是表".into());
            }
        }

        if let Some(status_line) = Self::status_line_item(&document) {
            let array = status_line
                .as_array()
                .ok_or("Codex 的 tui.status_line 必须是字符串数组")?;
            if array.iter().any(|item| item.as_str().is_none()) {
                return Err("Codex 的 tui.status_line 只能包含字符串".into());
            }

            if Self::is_recommended_status_line(status_line) {
                println!("✓ Codex tui.status_line 已是推荐配置");
                return Ok(());
            }
        }

        if existed {
            let backup_path = Self::backup_existing_file(path)?;
            println!("📦 已备份 Codex 配置: {}", backup_path.display());
        } else if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        if document.get("tui").is_none() {
            document["tui"] = Item::Table(Table::new());
        }

        let tui = document
            .get_mut("tui")
            .and_then(Item::as_table_mut)
            .ok_or("Codex 配置中的 tui 必须是表")?;

        let mut status_line = Array::new();
        for item in RECOMMENDED_STATUS_LINE {
            status_line.push(*item);
        }
        tui["status_line"] = Item::Value(Value::Array(status_line));

        atomic_write(path, document.to_string().as_bytes())?;
        println!("✓ Codex tui.status_line 配置完成: {}", path.display());
        Ok(())
    }

    /// 验证 Codex 配置。
    pub fn check() -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::get_config_path().ok_or("无法找到 Codex 配置路径")?;
        Self::check_at(&path)
    }

    /// 验证指定路径的 Codex 配置。
    pub fn check_at(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if !path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(path)?;
        let document = content.parse::<DocumentMut>()?;
        if let Some(status_line) = Self::status_line_item(&document) {
            let array = status_line
                .as_array()
                .ok_or("Codex 的 tui.status_line 必须是字符串数组")?;
            if array.iter().any(|item| item.as_str().is_none()) {
                return Err("Codex 的 tui.status_line 只能包含字符串".into());
            }
        }
        Ok(())
    }

    /// 打印当前 Codex 原生配置。
    pub fn print() -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::get_config_path().ok_or("无法找到 Codex 配置路径")?;
        Self::print_at(&path)
    }

    /// 打印指定路径的 Codex 原生配置。
    pub fn print_at(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if !path.exists() {
            println!("Codex 配置不存在: {}", path.display());
            return Ok(());
        }

        let content = fs::read_to_string(path)?;
        let document = content.parse::<DocumentMut>()?;
        println!("Codex 配置: {}", path.display());
        match Self::status_line_item(&document) {
            Some(status_line) => {
                let array = status_line
                    .as_array()
                    .ok_or("Codex 的 tui.status_line 必须是字符串数组")?;
                println!("tui.status_line = {array}");
            }
            None => println!("tui.status_line 未配置"),
        }
        Ok(())
    }

    fn status_line_item(document: &DocumentMut) -> Option<&Item> {
        document
            .get("tui")
            .and_then(Item::as_table)
            .and_then(|table| table.get("status_line"))
    }

    fn is_recommended_status_line(item: &Item) -> bool {
        let Some(array) = item.as_array() else {
            return false;
        };

        array.len() == RECOMMENDED_STATUS_LINE.len()
            && array
                .iter()
                .zip(RECOMMENDED_STATUS_LINE)
                .all(|(actual, expected)| actual.as_str() == Some(*expected))
    }

    fn backup_existing_file(path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let parent = path.parent().ok_or("无法确定 Codex 配置备份目录")?;
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("无法确定 Codex 配置文件名")?;
        let offset = FixedOffset::east_opt(8 * 60 * 60).ok_or("无法创建北京时间偏移")?;
        let timestamp = Utc::now()
            .with_timezone(&offset)
            .format("%Y%m%d-%H%M%S")
            .to_string();

        let mut backup_path = parent.join(format!("{file_name}.backup.{timestamp}"));
        let mut suffix = 1;
        while backup_path.exists() {
            backup_path = parent.join(format!("{file_name}.backup.{timestamp}.{suffix}"));
            suffix += 1;
        }

        fs::copy(path, &backup_path)?;
        Ok(backup_path)
    }
}

fn atomic_write(path: &Path, content: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let parent = path.parent().ok_or("无法确定 Codex 配置目录")?;
    fs::create_dir_all(parent)?;
    let existing_permissions = fs::metadata(path)
        .ok()
        .map(|metadata| metadata.permissions());

    let nonce = Utc::now().timestamp_nanos_opt().unwrap_or_default();
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("无法确定 Codex 配置文件名")?;
    let temp_path = parent.join(format!(".{file_name}.tmp-{}-{nonce}", std::process::id()));

    fs::write(&temp_path, content)?;
    if let Some(permissions) = existing_permissions {
        fs::set_permissions(&temp_path, permissions)?;
    }
    let result = replace_file(&temp_path, path);
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

fn replace_file(temp_path: &Path, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(windows)]
    {
        if path.exists() {
            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or("无法确定 Codex 配置文件名")?;
            let nonce = Utc::now().timestamp_nanos_opt().unwrap_or_default();
            let old_path =
                path.with_file_name(format!(".{file_name}.old-{}-{nonce}", std::process::id(),));
            fs::rename(path, &old_path)?;

            return match fs::rename(temp_path, path) {
                Ok(()) => {
                    let _ = fs::remove_file(old_path);
                    Ok(())
                }
                Err(error) => {
                    let _ = fs::rename(old_path, path);
                    Err(error.into())
                }
            };
        }
    }

    fs::rename(temp_path, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_config_path() -> (PathBuf, PathBuf) {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("byebyecode-codex-{nonce}"));
        (dir.clone(), dir.join("config.toml"))
    }

    #[test]
    fn overwrites_status_line_and_preserves_other_settings() {
        let (dir, path) = temp_config_path();
        fs::create_dir_all(&dir).expect("create temp directory");
        fs::write(&path, "model = \"gpt-5.6\"\n\n[tui]\nstatus_line = []\n").expect("write config");

        CodexConfigurator::configure_statusline_at(&path).expect("configure Codex");
        let document = fs::read_to_string(&path)
            .expect("read configured file")
            .parse::<DocumentMut>()
            .expect("parse configured file");
        assert_eq!(document["model"].as_str(), Some("gpt-5.6"));
        assert!(CodexConfigurator::is_recommended_status_line(
            &document["tui"]["status_line"]
        ));
        assert!(fs::read_dir(&dir)
            .expect("read temp directory")
            .filter_map(Result::ok)
            .any(|entry| entry.file_name().to_string_lossy().contains("backup")));

        fs::remove_dir_all(dir).expect("remove temp directory");
    }

    #[test]
    fn rejects_non_array_status_line_without_modifying_file() {
        let (dir, path) = temp_config_path();
        fs::create_dir_all(&dir).expect("create temp directory");
        let original = "[tui]\nstatus_line = \"invalid\"\n";
        fs::write(&path, original).expect("write config");

        assert!(CodexConfigurator::configure_statusline_at(&path).is_err());
        assert_eq!(fs::read_to_string(&path).expect("read config"), original);
        assert_eq!(fs::read_dir(&dir).expect("read temp directory").count(), 1);

        fs::remove_dir_all(dir).expect("remove temp directory");
    }

    #[test]
    fn handles_existing_nested_tui_table() {
        let (dir, path) = temp_config_path();
        fs::create_dir_all(&dir).expect("create temp directory");
        fs::write(&path, "[tui.model_availability_nux]\n\"gpt-5.6\" = 1\n").expect("write config");

        CodexConfigurator::configure_statusline_at(&path).expect("configure Codex");
        let document = fs::read_to_string(&path)
            .expect("read configured file")
            .parse::<DocumentMut>()
            .expect("parse configured file");
        assert_eq!(
            document["tui"]["model_availability_nux"]["gpt-5.6"].as_integer(),
            Some(1)
        );
        assert!(CodexConfigurator::is_recommended_status_line(
            &document["tui"]["status_line"]
        ));

        fs::remove_dir_all(dir).expect("remove temp directory");
    }
}
