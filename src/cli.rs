use clap::Parser;
use std::ffi::{OsStr, OsString};

use crate::target::Target;

#[derive(Parser, Debug)]
#[command(name = "byebyecode")]
#[command(version, about = "byebyecode - Claude Code 与 Codex 兼容工具")]
pub struct Cli {
    /// 选择宿主应用
    #[arg(long = "target", default_value_t = Target::default())]
    pub target: Target,

    /// Enter TUI configuration mode
    #[arg(short = 'c', long = "config")]
    pub config: bool,

    /// Set theme
    #[arg(short = 't', long = "theme")]
    pub theme: Option<String>,

    /// Print current configuration
    #[arg(long = "print")]
    pub print: bool,

    /// Initialize config file
    #[arg(long = "init")]
    pub init: bool,

    /// Check configuration
    #[arg(long = "check")]
    pub check: bool,

    /// Check for updates
    #[arg(short = 'u', long = "update")]
    pub update: bool,

    /// Patch Claude Code cli.js to disable context warnings
    #[arg(long = "patch")]
    pub patch: Option<String>,

    /// 启动宿主应用并透传末尾参数
    #[arg(long = "wrap", conflicts_with_all = ["config", "print", "init", "check", "update", "patch"])]
    pub wrap: bool,

    /// `--wrap --` 之后要透传的参数
    #[arg(last = true, requires = "wrap")]
    pub command_args: Vec<String>,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse_from(Self::normalize_wrap_args(std::env::args_os()))
    }

    fn normalize_wrap_args<I, T>(args: I) -> Vec<OsString>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString>,
    {
        let mut args: Vec<OsString> = args.into_iter().map(Into::into).collect();
        if args.iter().any(|arg| arg == OsStr::new("--")) {
            return args;
        }

        let Some(wrap_index) = args.iter().position(|arg| arg == OsStr::new("--wrap")) else {
            return args;
        };

        let first_host_arg = wrap_index + 1;
        let insert_at = match args.get(first_host_arg).and_then(|arg| arg.to_str()) {
            Some("--target") => {
                let Some(value) = args.get(first_host_arg + 1).and_then(|arg| arg.to_str()) else {
                    return args;
                };
                if value.parse::<Target>().is_err() {
                    return args;
                }
                first_host_arg + 2
            }
            Some(arg) if arg.starts_with("--target=") => {
                let value = arg.trim_start_matches("--target=");
                if value.parse::<Target>().is_err() {
                    return args;
                }
                first_host_arg + 1
            }
            _ => first_host_arg,
        };

        args.insert(insert_at, OsString::from("--"));
        args
    }
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use crate::target::Target;
    use clap::Parser;

    #[test]
    fn defaults_to_claude_target() {
        let cli = Cli::try_parse_from(["byebyecode", "--check"]).unwrap();
        assert_eq!(cli.target, Target::Claude);
    }

    #[test]
    fn parses_codex_wrap_arguments() {
        let cli = Cli::try_parse_from([
            "byebyecode",
            "--wrap",
            "--target",
            "codex",
            "--",
            "--model",
            "gpt-5.6",
        ])
        .unwrap();
        assert_eq!(cli.target, Target::Codex);
        assert!(cli.wrap);
        assert_eq!(
            cli.command_args,
            vec!["--model".to_string(), "gpt-5.6".to_string()]
        );
    }

    #[test]
    fn preserves_explicit_wrap_delimiter() {
        let original = [
            "byebyecode",
            "--wrap",
            "--target",
            "codex",
            "--",
            "--version",
        ];
        let args = Cli::normalize_wrap_args(original);

        assert_eq!(
            args,
            original
                .into_iter()
                .map(std::ffi::OsString::from)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn does_not_normalize_arguments_without_wrap() {
        let original = ["byebyecode", "--check", "--target", "codex"];
        let args = Cli::normalize_wrap_args(original);

        assert_eq!(
            args,
            original
                .into_iter()
                .map(std::ffi::OsString::from)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn parses_wrap_arguments_when_powershell_removes_delimiter() {
        let args =
            Cli::normalize_wrap_args(["byebyecode", "--wrap", "--target", "codex", "--version"]);
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.target, Target::Codex);
        assert!(cli.wrap);
        assert_eq!(cli.command_args, vec!["--version".to_string()]);
    }

    #[test]
    fn preserves_target_option_for_wrapped_host() {
        let args = Cli::normalize_wrap_args([
            "byebyecode",
            "--wrap",
            "--target=codex",
            "--target",
            "host-profile",
        ]);
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.target, Target::Codex);
        assert_eq!(
            cli.command_args,
            vec!["--target".to_string(), "host-profile".to_string()]
        );
    }

    #[test]
    fn supports_target_before_wrap_without_delimiter() {
        let args =
            Cli::normalize_wrap_args(["byebyecode", "--target", "codex", "--wrap", "--help"]);
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.target, Target::Codex);
        assert_eq!(cli.command_args, vec!["--help".to_string()]);
    }

    #[test]
    fn rejects_invalid_wrapper_target_without_delimiter() {
        let args = Cli::normalize_wrap_args([
            "byebyecode",
            "--wrap",
            "--target",
            "unsupported",
            "--version",
        ]);

        assert!(Cli::try_parse_from(args).is_err());
    }

    #[test]
    fn rejects_patch_for_wrap() {
        assert!(Cli::try_parse_from(["byebyecode", "--wrap", "--patch", "file"]).is_err());
    }

    #[test]
    fn rejects_trailing_arguments_without_wrap() {
        assert!(Cli::try_parse_from(["byebyecode", "--", "--model", "gpt-5.6"]).is_err());
    }
}
