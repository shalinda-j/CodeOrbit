﻿# Slash Commands Example Extension

This is an example extension showcasing how to write slash commands.

See: [Extensions: Slash Commands](https://CodeOrbit.dev/docs/extensions/slash-commands) in the CodeOrbit Docs.

## Pre-requisites

[Install Rust Toolchain](https://www.rust-lang.org/tools/install):

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Setup

```sh
git clone https://github.com/CodeOrbit-industries/CodeOrbit.git
cp -RL CodeOrbit/extensions/slash-commands-example .

cd slash-commands-example/

# Update Cargo.toml to make it standalone
cat > Cargo.toml << EOF
[package]
name = "slash_commands_example"
version = "0.1.0"
edition = "2021"
license = "Apache-2.0"

[lib]
path = "src/slash_commands_example.rs"
crate-type = ["cdylib"]

[dependencies]
codeorbit_extension_api = "0.1.0"
EOF

curl -O https://raw.githubusercontent.com/rust-lang/rust/master/LICENSE-APACHE
echo "# CodeOrbit Slash Commands Example Extension" > README.md
echo "Cargo.lock" > .gitignore
echo "target/" >> .gitignore
echo "*.wasm" >> .gitignore

git init
git add .
git commit -m "Initial commit"

cd ..
mv slash-commands-example MY-SUPER-COOL-CodeOrbit-EXTENSION
CodeOrbit $_
```

## Installation

1. Open the command palette (`cmd-shift-p` or `ctrl-shift-p`).
2. Launch `CodeOrbit: install dev extension`
3. Select the extension folder created above

## Test

Open the assistant and type `/echo` and `/pick-one` at the beginning of a line.

## Customization

Open the `extensions.toml` file and set the `id`, `name`, `description`, `authors` and `repository` fields.

Rename `slash-commands-example.rs` you'll also have to update `Cargo.toml`

## Rebuild

Rebuild to see these changes reflected:

1. Open CodeOrbit Extensions (`cmd-shift-x` or `ctrl-shift-x`).
2. Click `Rebuild` next to your Dev Extension (formerly "Slash Command Example")

## Troubleshooting / Logs

- MacOS: `tail -f ~/Library/Logs/CodeOrbit/CodeOrbit.log`
- Linux: `tail -f ~/.local/share/CodeOrbit/logs/CodeOrbit.log`

## Documentation

- [CodeOrbit.dev docs: Extensions: Developing Extensions](https://CodeOrbit.dev/docs/extensions/developing-extensions)
- [CodeOrbit.dev docs: Extensions: Slash Commands](https://CodeOrbit.dev/docs/extensions/slash-commands)
