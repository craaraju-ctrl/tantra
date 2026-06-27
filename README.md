# 🧠 Tantra — Engineering Loop Orchestrator

**Tantra** is an intelligent engineering loop agent that autonomously plans, implements, tests, reviews, and learns from coding tasks. It delegates implementation to **OpenHands** (AI coding sub-agent) and stores lessons in **agentic-memory** for continuous improvement.

```
  Plan → Code (OpenHands) → Test → Review → Learn
```

---

## Features

- **Engineering Loop** — Full Plan → Code → Test → Review → Learn cycle
- **OpenHands Integration** — Delegates coding tasks to OpenHands CLI
- **Memory Persistence** — Stores lessons, plans, and design knowledge in agentic-memory
- **CLI Dashboard** — Status, health, search across engineering memory
- **Launchd Auto-Restart** — Managed as a macOS service with KeepAlive
- **Binary Auto-Sync** — WatchPaths auto-copies after `cargo build --release`

## Quick Start

```bash
# Prerequisites
cargo build --release
cp target/release/tantra ~/.local/bin/

# Check OpenHands is available
tantra check-openhands

# Run the engineering loop
tantra eng-loop "Add error handling to the API client"
```

## Commands

| Command | Description |
|---------|-------------|
| `tantra status` | Show memory stats from agentic-memory |
| `tantra health` | Show system health (memory + OpenHands) |
| `tantra test` | Run integration tests against memory API |
| `tantra search <query>` | Smart search across engineering memory |
| `tantra eng-loop <task>` | Run full engineering loop (Plan → Code → Test → Review → Learn) |
| `tantra plan <task>` | Generate a plan for a task (dry-run) |
| `tantra seed-design` | Seed Tredo design system into memory |
| `tantra check-openhands` | Verify OpenHands is available |

## Architecture

```
┌──────────────────────────────────────────────────┐
│                   Tantra CLI                      │
│  ┌────────────────────────────────────────────┐   │
│  │          Engineering Loop                    │   │
│  │  ┌─────┐  ┌──────┐  ┌────┐  ┌──────┐  ┌──┐  │   │
│  │  │Plan │→│Code  │→│Test│→│Review│→│Learn│  │   │
│  │  └─────┘  └──────┘  └────┘  └──────┘  └──┘  │   │
│  └────────────────────────────────────────────┘   │
│         │           │              │                │
│         ▼           ▼              ▼                │
│  ┌──────────┐ ┌──────────┐ ┌────────────────┐      │
│  │ Memory   │ │OpenHands │ │ Cargo (check/   │      │
│  │ API      │ │ CLI      │ │ test)           │      │
│  └──────────┘ └──────────┘ └────────────────┘      │
└──────────────────────────────────────────────────┘
```

### Phases

1. **Plan** — Researches the task and generates a structured plan
2. **Code** — Delegates implementation to OpenHands CLI with the plan
3. **Test** — Runs `cargo check` and `cargo test` to verify
4. **Review** — Analyzes test results and decides if the loop continues
5. **Learn** — Stores outcomes and lessons in agentic-memory

## Configuration

Tantra reads environment variables from `~/.config/tantra/tantra.env`:

```bash
# Memory API server
MEMORY_API_URL=http://localhost:3111

# OpenHands CLI path
OPENHANDS_PATH=openhands

# Project directory (for cargo operations)
TANTRA_PROJECT_DIR=/path/to/project
```

## Installation

### macOS (launchd auto-start)

```bash
# Build and copy binary
cargo build --release
cp target/release/tantra ~/.local/bin/

# Copy env file
mkdir -p ~/.config/tantra
cp tantra.env.example ~/.config/tantra/tantra.env
# Edit ~/.config/tantra/tantra.env with your settings

# Load the launchd service
launchctl load ~/Library/LaunchAgents/com.tantra.loop.plist
```

The service runs `tantra health` every 60 seconds and auto-restarts on crash.

### Binary Auto-Sync

After each `cargo build --release`, the WatchPaths service automatically copies the binary to `~/.local/bin/tantra`.

## Dependencies

- **Rust** 1.70+ (edition 2021)
- **OpenHands CLI** — `pip install openhands`
- **agentic-memory** — Running on port 3111 (or configured `MEMORY_API_URL`)

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Check for issues
cargo clippy
cargo fmt --check

# Build release
cargo build --release
```

## Related Projects

- [Tredo](https://github.com/craaraju-ctrl/Tredo) — Agentic AI Trading System
- [Memory](https://github.com/craaraju-ctrl/memory) — Agentic Memory System
- [Knowledge](https://github.com/craaraju-ctrl/knowledge) — Web Research Agent
