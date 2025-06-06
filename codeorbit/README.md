﻿# CodeOrbit - AI-Powered Development Assistant for CodeOrbit

![CodeOrbit Logo](./assets/logo.codeorbit.svg)

CodeOrbit is an intelligent code assistant extension for CodeOrbit that provides AI-powered development tools, multi-agent collaboration, and context-aware coding assistance.

## Features

- ðŸš€ AI-Powered Code Completion & Generation
- ðŸ¤– Multi-Agent System for different development tasks
- ðŸ§  Context-Aware Development Environment
- ðŸ› ï¸ Built-in Development Tools & Utilities
- ðŸ”Œ Extensible Architecture for Custom Agents

## Installation

### Prerequisites

- [CodeOrbit Editor](https://CodeOrbit.dev/)
- Rust toolchain (latest stable)
- Cargo

### Building from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/your-org/codeorbit-CodeOrbit.git
   cd codeorbit-CodeOrbit/codeorbit
   ```

2. Build the extension:
   ```bash
   cargo build --release
   ```

3. Install the extension in CodeOrbit:
   - Open CodeOrbit
   - Open the command palette (Cmd/Ctrl+Shift+P)
   - Run "Install Extension"
   - Select the `target/release/libcodeorbit.so` (Linux), `target/release/libcodeorbit.dylib` (macOS), or `target/release/codeorbit.dll` (Windows) file

## Usage

Once installed, you can access CodeOrbit features through:

- **Command Palette**: Press `Cmd/Ctrl+Shift+P` and search for "CodeOrbit"
- **Keyboard Shortcut**: `Ctrl+Shift+O` (configurable in CodeOrbit settings)
- **Context Menu**: Right-click in the editor for context-aware actions

## Configuration

CodeOrbit can be configured via the `config.toml` file. The following options are available:

```toml
[core]
enabled = true
log_level = "info"

[ai]
model = "gpt-4"
max_tokens = 2048
temperature = 0.7

[ui]
show_welcome = true
panel_position = "right"
panel_width = 400
panel_height = 300
```

## Development

### Project Structure

```
codeorbit/
â”œâ”€â”€ src/                 # Rust source code
â”œâ”€â”€ agents/              # Agent implementations
â”‚   â”œâ”€â”€ frontend/       # Frontend-related agents
â”‚   â”œâ”€â”€ backend/        # Backend-related agents
â”‚   â”œâ”€â”€ database/       # Database-related agents
â”‚   â”œâ”€â”€ devops/         # DevOps-related agents
â”‚   â””â”€â”€ docs/           # Documentation-related agents
â”œâ”€â”€ core/               # Core functionality
â”œâ”€â”€ ui/                 # UI components
â”œâ”€â”€ assets/             # Static assets
â”œâ”€â”€ Cargo.toml          # Rust project configuration
â”œâ”€â”€ CodeOrbit.toml            # CodeOrbit extension manifest
â””â”€â”€ config.toml         # Extension configuration
```

### Building and Testing

```bash
# Build in debug mode
cargo build

# Build in release mode
cargo build --release

# Run tests
cargo test

# Format code
cargo fmt

# Check for clippy warnings
cargo clippy
```

## Prompt Handling Loop

The extension provides a simple round-trip for user prompts. A prompt entered in
the UI is sent to the orchestrator, which forwards it to the `UiPlannerAgent`.
The agent returns a UI component plan and the orchestrator delivers this back to
the prompt panel for display.

Submit a prompt by pressing **Enter** inside the panel's input area or by
clicking the *Send* button. Any agent errors are shown inline.

Main files involved:

- `ui/prompt_panel.rs` â€“ gathers user input and renders responses.
- `core/orchestrator.rs` â€“ routes prompts and manages agents.
- `agents/frontend/ui_planner_agent.rs` â€“ interprets prompts and creates a UI plan.


## Contributing

Contributions are welcome! Please read our [Contributing Guidelines](CONTRIBUTING.md) for details.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

For support, please open an issue in our issue tracker or join our community chat.
