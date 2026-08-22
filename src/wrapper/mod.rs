use crate::target::Target;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};

/// Find Claude Code executable from PATH environment variable
pub fn find_claude_code() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Try to find 'claude' command in PATH
    match which::which("claude") {
        Ok(path) => Ok(path),
        Err(_) => {
            // Try common installation paths on different platforms
            #[cfg(target_os = "windows")]
            {
                // Windows: Check AppData locations
                if let Ok(appdata) = std::env::var("APPDATA") {
                    let claude_path = PathBuf::from(appdata).join("npm").join("claude.cmd");
                    if claude_path.exists() {
                        return Ok(claude_path);
                    }
                }
            }

            #[cfg(target_os = "macos")]
            {
                // macOS: Check common npm global paths
                let paths = vec!["/usr/local/bin/claude", "/opt/homebrew/bin/claude"];
                for path in paths {
                    let p = PathBuf::from(path);
                    if p.exists() {
                        return Ok(p);
                    }
                }
            }

            #[cfg(target_os = "linux")]
            {
                // Linux: Check common paths
                let paths = vec!["/usr/local/bin/claude", "/usr/bin/claude"];
                for path in paths {
                    let p = PathBuf::from(path);
                    if p.exists() {
                        return Ok(p);
                    }
                }
            }

            Err("Claude Code executable not found in PATH or common locations".into())
        }
    }
}

/// 从当前环境查找支持的宿主可执行文件。
pub fn find_target(target: Target) -> Result<PathBuf, Box<dyn std::error::Error>> {
    match target {
        Target::Claude => find_claude_code(),
        Target::Codex => find_codex(),
        Target::Both => Err("Both target is not supported for wrapper mode".into()),
    }
}

/// 查找 Codex 可执行文件，包括 Windows npm 命令 shim。
pub fn find_codex() -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Ok(path) = which::which("codex") {
        return Ok(path);
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            let path = PathBuf::from(appdata).join("npm").join("codex.cmd");
            if path.exists() {
                return Ok(path);
            }
        }
    }

    Err("Codex executable not found in PATH or common locations".into())
}

/// 启动宿主应用并原样透传所有参数。
pub fn run_target(
    target: Target,
    args: &[String],
) -> Result<ExitStatus, Box<dyn std::error::Error>> {
    let executable = find_target(target)?;
    let mut command;

    #[cfg(windows)]
    {
        if executable
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.eq_ignore_ascii_case("cmd"))
            .unwrap_or(false)
        {
            command = Command::new("cmd");
            command.arg("/c").arg(&executable);
        } else {
            command = Command::new(&executable);
        }
    }

    #[cfg(not(windows))]
    {
        command = Command::new(&executable);
    }

    Ok(command.args(args).status()?)
}
