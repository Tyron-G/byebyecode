use clap::Parser;

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
        Self::parse()
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
    fn rejects_patch_for_wrap() {
        assert!(Cli::try_parse_from(["byebyecode", "--wrap", "--patch", "file"]).is_err());
    }

    #[test]
    fn rejects_trailing_arguments_without_wrap() {
        assert!(Cli::try_parse_from(["byebyecode", "--", "--model", "gpt-5.6"]).is_err());
    }
}
