use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Target {
    #[default]
    Claude,
    Codex,
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Claude => write!(f, "claude"),
            Self::Codex => write!(f, "codex"),
        }
    }
}

impl std::str::FromStr for Target {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "claude" => Ok(Self::Claude),
            "codex" => Ok(Self::Codex),
            _ => Err(format!("不支持的目标: {value}，可选值为 claude 或 codex")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Target;
    use std::str::FromStr;

    #[test]
    fn parses_supported_targets_case_insensitively() {
        assert_eq!(Target::from_str("claude").unwrap(), Target::Claude);
        assert_eq!(Target::from_str("CODEX").unwrap(), Target::Codex);
    }

    #[test]
    fn rejects_unknown_target() {
        assert!(Target::from_str("unknown").is_err());
    }
}
