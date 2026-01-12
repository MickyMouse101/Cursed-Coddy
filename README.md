# Cursed Coddy

A CLI-based coding education platform for learning programming through interactive lessons.

## Prerequisites

- Rust (latest stable version)
- Ollama (for AI-powered lesson generation)

### Installing Rust

**Linux and macOS:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Windows:**
Download and run the installer from [rustup.rs](https://rustup.rs/)

After installation, restart your terminal or run:
```bash
source $HOME/.cargo/env  # Linux/macOS
```

### Installing Ollama

**Linux:**
```bash
curl -fsSL https://ollama.com/install.sh | sh
```

**macOS:**
```bash
brew install ollama
```

**Windows:**
Download the installer from [ollama.com](https://ollama.com/download)

After installation, start the Ollama service:
```bash
ollama serve
```

## Building from Source

1. Clone the repository:
```bash
git clone https://github.com/yourusername/Cursed-Coddy.git
cd Cursed-Coddy
```

2. Build the project:
```bash
cargo build --release
```

The executable will be located at `target/release/cursed-coddy`

## Installation

After building, install the binary to your system:

```bash
cargo install --path .
```

This will install `cursed-coddy` to `~/.cargo/bin/` (or `%USERPROFILE%\.cargo\bin\` on Windows).

Make sure this directory is in your PATH.

## Usage

Start a new lesson:
```bash
cursed-coddy start
```

Continue from where you left off:
```bash
cursed-coddy continue
```

Start or continue a learning journey:
```bash
cursed-coddy journey
```

View your progress:
```bash
cursed-coddy progress
```

Show help:
```bash
cursed-coddy help
```

## Supported Languages

- Rust
- JavaScript
- C++

## License

Apache-2.0
