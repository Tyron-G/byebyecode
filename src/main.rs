use byebyecode::cli::Cli;
use byebyecode::config::{Config, InputData};
use byebyecode::core::{collect_all_segments, StatusLineGenerator};
use byebyecode::target::Target;
use std::io::{self, IsTerminal};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    migrate_legacy_config()?;

    let cli = Cli::parse_args();

    if cli.wrap {
        let status = byebyecode::wrapper::run_target(cli.target, &cli.command_args)?;
        return exit_with_status(status);
    }

    match cli.target {
        Target::Both => run_both(cli),
        Target::Claude => run_claude(cli),
        Target::Codex => run_codex(cli),
    }
}

fn run_both(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    if cli.config {
        #[cfg(feature = "tui")]
        {
            println!("正在同时配置 Claude 和 Codex...");
            let mut errors = Vec::new();
            
            if let Err(e) = run_claude_cli_only() {
                errors.push(format!("Claude 配置失败: {}", e));
            }
            if let Err(e) = run_codex_cli_only() {
                errors.push(format!("Codex 配置失败: {}", e));
            }
            
            if !errors.is_empty() {
                for error in errors {
                    eprintln!("⚠ {}", error);
                }
                return Err("部分配置失败".into());
            }
            println!("✓ Claude 和 Codex 配置完成");
        }
        #[cfg(not(feature = "tui"))]
        {
            eprintln!("TUI feature is not enabled. Please install with --features tui");
            std::process::exit(1);
        }
        return Ok(());
    }
    
    // 其他操作按需要分发给两个目标
    let claude_cli = cli.clone();
    let codex_cli = cli.clone();
    
    let mut errors = Vec::new();
    
    if let Err(e) = run_claude(claude_cli) {
        errors.push(format!("Claude 操作失败: {}", e));
    }
    if let Err(e) = run_codex(codex_cli) {
        errors.push(format!("Codex 操作失败: {}", e));
    }
    
    if !errors.is_empty() {
        for error in errors {
            eprintln!("⚠ {}", error);
        }
        return Err("部分操作失败".into());
    }
    
    Ok(())
}

fn run_claude_cli_only() -> Result<(), Box<dyn std::error::Error>> {
    byebyecode::ui::run_configurator(Target::Claude)
}

fn run_codex_cli_only() -> Result<(), Box<dyn std::error::Error>> {
    byebyecode::ui::run_configurator(Target::Codex)
}

fn run_codex(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    validate_codex_cli(&cli)?;

    if cli.init {
        Config::init_for_target(Target::Codex)?;
        byebyecode::auto_config::CodexConfigurator::configure_statusline()?;
        return Ok(());
    }

    if cli.print {
        let mut config =
            Config::load_for_target(Target::Codex).unwrap_or_else(|_| Config::default());
        if let Some(theme) = cli.theme.as_deref() {
            config = byebyecode::ui::themes::ThemePresets::get_theme(theme);
        }
        config.print()?;
        byebyecode::auto_config::CodexConfigurator::print()?;
        return Ok(());
    }

    if cli.check {
        byebyecode::wrapper::find_codex()?;
        byebyecode::auto_config::CodexConfigurator::check()?;
        let config = Config::load_for_target(Target::Codex)?;
        config.check()?;
        println!("✓ Codex 环境和配置有效");
        return Ok(());
    }

    if cli.config {
        #[cfg(feature = "tui")]
        {
            byebyecode::ui::run_configurator(Target::Codex)?;
        }
        #[cfg(not(feature = "tui"))]
        {
            eprintln!("TUI feature is not enabled. Please install with --features tui");
            std::process::exit(1);
        }
        return Ok(());
    }

    if cli.update {
        run_update();
        return Ok(());
    }

    Err("未指定有效的 Codex 操作".into())
}

fn run_claude(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    if cli.init {
        Config::init()?;

        println!("\n正在配置 Claude Code settings.json...");
        match byebyecode::auto_config::ClaudeSettingsConfigurator::configure_statusline() {
            Ok(_) => {}
            Err(e) => {
                eprintln!("⚠ 配置 Claude settings.json 失败: {}", e);
                eprintln!("  你可以手动配置 statusLine 字段");
            }
        }

        return Ok(());
    }

    if cli.print {
        let mut config = Config::load().unwrap_or_else(|_| Config::default());
        if let Some(theme) = cli.theme.as_deref() {
            config = byebyecode::ui::themes::ThemePresets::get_theme(theme);
        }
        config.print()?;
        return Ok(());
    }

    if cli.check {
        let config = Config::load()?;
        config.check()?;
        println!("✓ Configuration valid");
        return Ok(());
    }

    if cli.config {
        #[cfg(feature = "tui")]
        {
            byebyecode::ui::run_configurator(Target::Claude)?;
        }
        #[cfg(not(feature = "tui"))]
        {
            eprintln!("TUI feature is not enabled. Please install with --features tui");
            std::process::exit(1);
        }
        return Ok(());
    }

    if cli.update {
        run_update();
        return Ok(());
    }

    if let Some(claude_path) = cli.patch {
        use byebyecode::utils::ClaudeCodePatcher;

        println!("🔧 Claude Code Context Warning Disabler");
        println!("Target file: {}", claude_path);

        let backup_path = format!("{}.backup", claude_path);
        std::fs::copy(&claude_path, &backup_path)?;
        println!("📦 Created backup: {}", backup_path);

        let mut patcher = ClaudeCodePatcher::new(&claude_path)?;
        println!("\n🔄 Applying patches...");

        if let Err(e) = patcher.write_verbose_property(true) {
            println!("⚠️ Could not modify verbose property: {}", e);
        }
        patcher.disable_context_low_warnings()?;
        if let Err(e) = patcher.disable_esc_interrupt_display() {
            println!("⚠️ Could not disable esc/interrupt display: {}", e);
        }
        if let Err(e) = patcher.add_statusline_refresh_interval(30000) {
            println!("⚠️ Could not add statusline auto-refresh: {}", e);
        }

        patcher.save()?;

        println!("✅ All patches applied successfully!");
        println!("💡 To restore warnings, replace your cli.js with the backup file:");
        println!("   cp {} {}", backup_path, claude_path);

        return Ok(());
    }

    let mut config = Config::load().unwrap_or_else(|_| Config::default());
    if let Some(theme) = cli.theme.as_deref() {
        config = byebyecode::ui::themes::ThemePresets::get_theme(theme);
    }

    if io::stdin().is_terminal() {
        #[cfg(feature = "tui")]
        {
            use byebyecode::ui::{MainMenu, MenuResult};

            if let Some(result) = MainMenu::run()? {
                match result {
                    MenuResult::LaunchConfigurator => {
                        byebyecode::ui::run_configurator(Target::Claude)?;
                    }
                    MenuResult::InitConfig => {
                        Config::init()?;
                        println!("Configuration initialized successfully!");
                    }
                    MenuResult::CheckConfig => {
                        let config = Config::load()?;
                        config.check()?;
                        println!("Configuration is valid!");
                    }
                    MenuResult::Exit => {}
                }
            }
        }
        #[cfg(not(feature = "tui"))]
        {
            eprintln!("No input data provided and TUI feature is not enabled.");
            eprintln!("Usage: echo '{{...}}' | byebyecode");
            eprintln!("   or: byebyecode --help");
        }
        return Ok(());
    }

    let stdin = io::stdin();
    let input: InputData = serde_json::from_reader(stdin.lock())?;
    let segments_data = collect_all_segments(&config, &input);
    let generator = StatusLineGenerator::new(config);
    println!("{}", generator.generate(segments_data));

    Ok(())
}

fn run_update() {
    #[cfg(feature = "self-update")]
    {
        println!("Update feature not implemented in new architecture yet");
    }
    #[cfg(not(feature = "self-update"))]
    {
        println!("Update check not available (self-update feature disabled)");
    }
}

fn validate_codex_cli(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    if cli.patch.is_some() {
        return Err("--patch 仅支持 Claude Code，Codex 不支持修改安装文件".into());
    }
    if cli.command_args.is_empty()
        && !cli.init
        && !cli.print
        && !cli.check
        && !cli.config
        && !cli.update
    {
        return Err("Codex 目标需要 --init、--check、--print、--config、--update 或 --wrap".into());
    }
    Ok(())
}

fn exit_with_status(status: std::process::ExitStatus) -> Result<(), Box<dyn std::error::Error>> {
    match status.code() {
        Some(code) => std::process::exit(code),
        None => Err("目标进程未返回退出码".into()),
    }
}

fn migrate_legacy_config() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(home) = dirs::home_dir() {
        let old_dir = home.join(".claude").join("88code");
        let new_dir = home.join(".claude").join("byebyecode");

        if old_dir.exists() && !new_dir.exists() {
            std::fs::rename(&old_dir, &new_dir)?;
            println!("✓ 已自动迁移配置目录: ~/.claude/88code → ~/.claude/byebyecode");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_codex_cli;
    use byebyecode::cli::Cli;
    use clap::Parser;

    #[test]
    fn codex_config_and_update_are_valid_operations() {
        for args in [
            ["byebyecode", "--config", "--target", "codex"],
            ["byebyecode", "--update", "--target", "codex"],
        ] {
            let cli = Cli::try_parse_from(args).unwrap();
            assert!(validate_codex_cli(&cli).is_ok());
        }
    }
}
