# @88code/byebyecode

Claude Code 状态栏与 Codex 兼容工具

## Installation

```bash
npm install -g @88code/byebyecode
```

## Features

- 🚀 **Fast**: Written in Rust for maximum performance
- 🌍 **Cross-platform**: Works on Windows, macOS, and Linux
- 📦 **Easy installation**: One command via npm
- 🔄 **Auto-update**: Built-in update notifications
- 🎨 **Beautiful**: Nerd Font icons and colors
- 🤖 **Codex**: 使用 Codex 原生 `tui.status_line` 配置

## Usage

After installation, byebyecode can configure Claude Code at `~/.claude/byebyecode`.

You can also use it directly:

```bash
byebyecode --help
byebyecode --version

# Codex
byebyecode --init --target codex
byebyecode --wrap --target codex -- --model gpt-5.6
```

## For Users in China

Use npm mirror for faster installation:

```bash
npm install -g @88code/byebyecode --registry https://registry.npmmirror.com
```

## More Information

- GitHub: https://github.com/Haleclipse/CCometixLine
- Issues: https://github.com/Haleclipse/CCometixLine/issues
- License: MIT
