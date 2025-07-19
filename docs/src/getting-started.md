# Getting Started

Welcome to CodeOrbit! We are excited to have you. Here is a jumping-off point to getting started.

## Download CodeOrbit

### macOS

Get the latest stable builds via [the download page](https://codeorbit.dev/download). If you want to download our preview build, you can find it on its [releases page](https://codeorbit.dev/releases/preview). After the first manual installation, CodeOrbit will periodically check for updates.

You can also install CodeOrbit stable via Homebrew:

```sh
brew install --cask codeorbit
```

### Linux

For most Linux users, the easiest way to install CodeOrbit is through our installation script:

```sh
curl -f https://codeorbit.dev/install.sh | sh
```

### Windows

Windows support is currently in development. Please check our [downloads page](https://codeorbit.dev/download) for the latest updates.

If you'd like to help us test our new features, you can also install our preview build:

```sh
curl -f https://CodeOrbit.dev/install.sh | CODEORBIT_CHANNEL=preview sh
```

This script supports `x86_64` and `AArch64`, as well as common Linux distributions: Ubuntu, Arch, Debian, RedHat, CentOS, Fedora, and more.

If CodeOrbit is installed using this installation script, it can be uninstalled at any time by running the shell command `CodeOrbit --uninstall`. The shell will then prompt you whether you'd like to keep your preferences or delete them. After making a choice, you should see a message that CodeOrbit was successfully uninstalled.

If this script is insufficient for your use case, you run into problems running CodeOrbit, or there are errors in uninstalling CodeOrbit, please see our [Linux-specific documentation](./linux.md).

## Command Palette

The Command Palette is the main way to access pretty much any functionality that's available in CodeOrbit. Its keybinding is the first one you should make yourself familiar with. To open it, hit: {#kb command_palette::Toggle}.

![The opened Command Palette](https://CodeOrbit.dev/img/features/command-palette.jpg)

Try it! Open the Command Palette and type in `new file`. You should see the list of commands being filtered down to `workspace: new file`. Hit return and you end up with a new buffer.

Any time you see instructions that include commands of the form `CodeOrbit: ...` or `editor: ...` and so on that means you need to execute them in the Command Palette.

## CLI

CodeOrbit has a CLI, on Linux this should come with the distribution's CodeOrbit package (binary name can vary from distribution to distribution, `CodeOrbit` will be used later for brevity).
For macOS, the CLI comes in the same package with the editor binary, and could be installed into the system with the `cli: install` CodeOrbit command which will create a symlink to the `/usr/local/bin/CodeOrbit`.
It can also be built from source out of the `cli` crate in this repository.

Use `CodeOrbit --help` to see the full list of capabilities.
General highlights:

- Opening another empty CodeOrbit window: `CodeOrbit`

- Opening a file or directory in CodeOrbit: `CodeOrbit /path/to/entry` (use `-n` to open in the new window)

- Reading from stdin: `ps axf | CodeOrbit -`

- Starting CodeOrbit with logs in the terminal: `CodeOrbit --foreground`

- Uninstalling CodeOrbit and all its related files: `CodeOrbit --uninstall`

## Configure CodeOrbit

To open your custom settings to set things like fonts, formatting settings, per-language settings, and more, use the {#kb CodeOrbit::OpenSettings} keybinding.

To see all available settings, open the Command Palette with {#kb command_palette::Toggle} and search for `CodeOrbit: open default settings`.
You can also check them all out in the [Configuring CodeOrbit](./configuring-CodeOrbit.md) documentation.

## Configure AI in CodeOrbit

CodeOrbit smoothly integrates LLMs in multiple ways across the editor.
Visit [the AI overview page](./ai/overview.md) to learn how to quickly get started with LLMs on CodeOrbit.

## Set up your key bindings

To open your custom keymap to add your key bindings, use the {#kb CodeOrbit::OpenKeymap} keybinding.

To access the default key binding set, open the Command Palette with {#kb command_palette::Toggle} and search for "CodeOrbit: open default keymap". See [Key Bindings](./key-bindings.md) for more info.
