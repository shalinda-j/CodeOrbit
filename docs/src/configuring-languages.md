# Configuring supported languages

CodeOrbit offers powerful customization options for each programming language it supports. This guide will walk you through the various ways you can tailor your coding experience to your preferences and project requirements.

CodeOrbit's language support is built on two main technologies:

1. Tree-sitter: This handles syntax highlighting and structure-based features like the outline panel.
2. Language Server Protocol (LSP): This provides semantic features such as code completion and diagnostics.

These components work together to provide CodeOrbit's language capabilities.

In this guide, we'll cover:

- Language-specific settings
- File associations
- Working with language servers
- Formatting and linting configuration
- Customizing syntax highlighting and themes
- Advanced language features

By the end of this guide, you should know how to configure and customize supported languages in CodeOrbit.

For a comprehensive list of languages supported by CodeOrbit and their specific configurations, see our [Supported Languages](./languages.md) page. To go further, you could explore developing your own extensions to add support for additional languages or enhance existing functionality. For more information on creating language extensions, see our [Language Extensions](./extensions/languages.md) guide.

## Language-specific Settings

CodeOrbit allows you to override global settings for individual languages. These custom configurations are defined in your `settings.json` file under the `languages` key.

Here's an example of language-specific settings:

```json
"languages": {
  "Python": {
    "tab_size": 4,
    "formatter": "language_server",
    "format_on_save": "on"
  },
  "JavaScript": {
    "tab_size": 2,
    "formatter": {
      "external": {
        "command": "prettier",
        "arguments": ["--stdin-filepath", "{buffer_path}"]
      }
    }
  }
}
```

You can customize a wide range of settings for each language, including:

- [`tab_size`](./configuring-CodeOrbit.md#tab-size): The number of spaces for each indentation level
- [`formatter`](./configuring-CodeOrbit.md#formatter): The tool used for code formatting
- [`format_on_save`](./configuring-CodeOrbit.md#format-on-save): Whether to automatically format code when saving
- [`enable_language_server`](./configuring-CodeOrbit.md#enable-language-server): Toggle language server support
- [`hard_tabs`](./configuring-CodeOrbit.md#hard-tabs): Use tabs instead of spaces for indentation
- [`preferred_line_length`](./configuring-CodeOrbit.md#preferred-line-length): The recommended maximum line length
- [`soft_wrap`](./configuring-CodeOrbit.md#soft-wrap): How to wrap long lines of code
- [`show_completions_on_input`](./configuring-CodeOrbit.md#show-completions-on-input): Whether or not to show completions as you type
- [`show_completion_documentation`](./configuring-CodeOrbit.md#show-completion-documentation): Whether to display inline and alongside documentation for items in the completions menu

These settings allow you to maintain specific coding styles across different languages and projects.

## File Associations

CodeOrbit automatically detects file types based on their extensions, but you can customize these associations to fit your workflow.

To set up custom file associations, use the [`file_types`](./configuring-CodeOrbit.md#file-types) setting in your `settings.json`:

```json
"file_types": {
  "C++": ["c"],
  "TOML": ["MyLockFile"],
  "Dockerfile": ["Dockerfile*"]
}
```

This configuration tells CodeOrbit to:

- Treat `.c` files as C++ instead of C
- Recognize files named "MyLockFile" as TOML
- Apply Dockerfile syntax to any file starting with "Dockerfile"

You can use glob patterns for more flexible matching, allowing you to handle complex naming conventions in your projects.

## Working with Language Servers

Language servers are a crucial part of CodeOrbit's intelligent coding features, providing capabilities like auto-completion, go-to-definition, and real-time error checking.

### What are Language Servers?

Language servers implement the Language Server Protocol (LSP), which standardizes communication between the editor and language-specific tools. This allows CodeOrbit to support advanced features for multiple programming languages without implementing each feature separately.

Some key features provided by language servers include:

- Code completion
- Error checking and diagnostics
- Code navigation (go to definition, find references)
- Code actions (Rename, extract method)
- Hover information
- Workspace symbol search

### Managing Language Servers

CodeOrbit simplifies language server management for users:

1. Automatic Download: When you open a file with a matching file type, CodeOrbit automatically downloads the appropriate language server. CodeOrbit may prompt you to install an extension for known file types.

2. Storage Location:

   - macOS: `~/Library/Application Support/CodeOrbit/languages`
   - Linux: `$XDG_DATA_HOME/languages`, `$FLATPAK_XDG_DATA_HOME/languages`, or `$HOME/.local/share`

3. Automatic Updates: CodeOrbit keeps your language servers up-to-date, ensuring you always have the latest features and improvements.

### Choosing Language Servers

Some languages in CodeOrbit offer multiple language server options. You might have multiple extensions installed that bundle language servers targeting the same language, potentially leading to overlapping capabilities. To ensure you get the functionality you prefer, CodeOrbit allows you to prioritize which language servers are used and in what order.

You can specify your preference using the `language_servers` setting:

```json
  "languages": {
    "PHP": {
      "language_servers": ["intelephense", "!phpactor", "..."]
    }
  }
```

In this example:

- `intelephense` is set as the primary language server
- `phpactor` is disabled (note the `!` prefix)
- `...` expands to the rest of the language servers that are registered for PHP

This configuration allows you to tailor the language server setup to your specific needs, ensuring that you get the most suitable functionality for your development workflow.

### Configuring Language Servers

Many language servers accept custom configuration options. You can set these in the `lsp` section of your `settings.json`:

```json
  "lsp": {
    "rust-analyzer": {
      "initialization_options": {
        "check": {
          "command": "clippy"
        }
      }
    }
  }
```

This example configures the Rust Analyzer to use Clippy for additional linting when saving files.

#### Nested objects

When configuring language server options in CodeOrbit, it's important to use nested objects rather than dot-delimited strings. This is particularly relevant when working with more complex configurations. Let's look at a real-world example using the TypeScript language server:

Suppose you want to configure the following settings for TypeScript:

- Enable strict null checks
- Set the target ECMAScript version to ES2020

Here's how you would structure these settings in CodeOrbit's `settings.json`:

```json
"lsp": {
  "typescript-language-server": {
    "initialization_options": {
      // These are not supported (VSCode dotted style):
      // "preferences.strictNullChecks": true,
      // "preferences.target": "ES2020"
      //
      // These is correct (nested notation):
      "preferences": {
        "strictNullChecks": true,
        "target": "ES2020"
      },
    }
  }
}
```

#### Possible configuration options

Depending on how a particular language server is implemented, they may depend on different configuration options, both specified in the LSP.

- [initializationOptions](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#version_3_17_0)

Sent once during language server startup, requires server's restart to reapply changes.

For example, rust-analyzer and clangd rely on this way of configuring only.

```json
  "lsp": {
    "rust-analyzer": {
      "initialization_options": {
        "checkOnSave": false
      }
    }
  }
```

- [Configuration Request](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#workspace_configuration)

May be queried by the server multiple times.
Most of the servers would rely on this way of configuring only.

```json
"lsp": {
  "tailwindcss-language-server": {
    "settings": {
      "tailwindCSS": {
        "emmetCompletions": true,
      },
    }
  }
}
```

Apart of the LSP-related server configuration options, certain servers in CodeOrbit allow configuring the way binary is launched by CodeOrbit.

Language servers are automatically downloaded or launched if found in your path, if you wish to specify an explicit alternate binary you can specify that in settings:

```json
  "lsp": {
    "rust-analyzer": {
      "binary": {
        // Whether to fetch the binary from the internet, or attempt to find locally.
        "ignore_system_version": false,
        "path": "/path/to/langserver/bin",
        "arguments": ["--option", "value"],
        "env": {
          "FOO": "BAR"
        }
      }
    }
  }
```

### Enabling or Disabling Language Servers

You can toggle language server support globally or per-language:

```json
  "languages": {
    "Markdown": {
      "enable_language_server": false
    }
  }
```

This disables the language server for Markdown files, which can be useful for performance in large documentation projects. You can configure this globally in your `~/.CodeOrbit/settings.json` or inside a `.CodeOrbit/settings.json` in your project directory.

## Formatting and Linting

CodeOrbit provides support for code formatting and linting to maintain consistent code style and catch potential issues early.

### Configuring Formatters

CodeOrbit supports both built-in and external formatters. See [`formatter`](./configuring-CodeOrbit.md#formatter) docs for more. You can configure formatters globally or per-language in your `settings.json`:

```json
"languages": {
  "JavaScript": {
    "formatter": {
      "external": {
        "command": "prettier",
        "arguments": ["--stdin-filepath", "{buffer_path}"]
      }
    },
    "format_on_save": "on"
  },
  "Rust": {
    "formatter": "language_server",
    "format_on_save": "on"
  }
}
```

This example uses Prettier for JavaScript and the language server's formatter for Rust, both set to format on save.

To disable formatting for a specific language:

```json
"languages": {
  "Markdown": {
    "format_on_save": "off"
  }
}
```

### Setting Up Linters

Linting in CodeOrbit is typically handled by language servers. Many language servers allow you to configure linting rules:

```json
"lsp": {
  "eslint": {
    "settings": {
      "codeActionOnSave": {
        "rules": ["import/order"]
      }
    }
  }
}
```

This configuration sets up ESLint to organize imports on save for JavaScript files.

To run linter fixes automatically on save:

```json
"languages": {
  "JavaScript": {
    "code_actions_on_format": {
      "source.fixAll.eslint": true
    }
  }
}
```

### Integrating Formatting and Linting

CodeOrbit allows you to run both formatting and linting on save. Here's an example that uses Prettier for formatting and ESLint for linting JavaScript files:

```json
"languages": {
  "JavaScript": {
    "formatter": {
      "external": {
        "command": "prettier",
        "arguments": ["--stdin-filepath", "{buffer_path}"]
      }
    },
    "code_actions_on_format": {
      "source.fixAll.eslint": true
    },
    "format_on_save": "on"
  }
}
```

### Troubleshooting

If you encounter issues with formatting or linting:

1. Check CodeOrbit's log file for error messages (Use the command palette: `CodeOrbit: open log`)
2. Ensure external tools (formatters, linters) are correctly installed and in your PATH
3. Verify configurations in both CodeOrbit settings and language-specific config files (e.g., `.eslintrc`, `.prettierrc`)

## Syntax Highlighting and Themes

CodeOrbit offers customization options for syntax highlighting and themes, allowing you to tailor the visual appearance of your code.

### Customizing Syntax Highlighting

CodeOrbit uses Tree-sitter grammars for syntax highlighting. Override the default highlighting using the `experimental.theme_overrides` setting.

This example makes comments italic and changes the color of strings:

```json
"experimental.theme_overrides": {
  "syntax": {
    "comment": {
      "font_style": "italic"
    },
    "string": {
      "color": "#00AA00"
    }
  }
}
```

### Selecting and Customizing Themes

Change your theme:

1. Use the theme selector ({#kb theme_selector::Toggle})
2. Or set it in your `settings.json`:

```json
"theme": {
  "mode": "dark",
  "dark": "One Dark",
  "light": "GitHub Light"
}
```

Create custom themes by creating a JSON file in `~/.config/CodeOrbit/themes/`. CodeOrbit will automatically detect and make available any themes in this directory.

### Using Theme Extensions

CodeOrbit supports theme extensions. Browse and install theme extensions from the Extensions panel ({#kb CodeOrbit::Extensions}).

To create your own theme extension, refer to the [Developing Theme Extensions](./extensions/themes.md) guide.

## Using Language Server Features

### Inlay Hints

Inlay hints provide additional information inline in your code, such as parameter names or inferred types. Configure inlay hints in your `settings.json`:

```json
"inlay_hints": {
  "enabled": true,
  "show_type_hints": true,
  "show_parameter_hints": true,
  "show_other_hints": true
}
```

For language-specific inlay hint settings, refer to the documentation for each language.

### Code Actions

Code actions provide quick fixes and refactoring options. Access code actions using the `editor: Toggle Code Actions` command or by clicking the lightbulb icon that appears next to your cursor when actions are available.

### Go To Definition and References

Use these commands to navigate your codebase:

- `editor: Go to Definition` (<kbd>f12|f12</kbd>)
- `editor: Go to Type Definition` (<kbd>cmd-f12|ctrl-f12</kbd>)
- `editor: Find All References` (<kbd>shift-f12|shift-f12</kbd>)

### Rename Symbol

To rename a symbol across your project:

1. Place your cursor on the symbol
2. Use the `editor: Rename Symbol` command (<kbd>f2|f2</kbd>)
3. Enter the new name and press Enter

These features depend on the capabilities of the language server for each language.

When renaming a symbol that spans multiple files, CodeOrbit will open a preview in a multibuffer. This allows you to review all the changes across your project before applying them. To confirm the rename, simply save the multibuffer. If you decide not to proceed with the rename, you can undo the changes or close the multibuffer without saving.

### Hover Information

Use the `editor: Show Hover` command to display information about the symbol under the cursor. This often includes type information, documentation, and links to relevant resources.

### Workspace Symbol Search

The `workspace: Open Symbol` command allows you to search for symbols (functions, classes, variables) across your entire project. This is useful for quickly navigating large codebases.

### Code Completion

CodeOrbit provides intelligent code completion suggestions as you type. You can manually trigger completion with the `editor: Show Completions` command. Use <kbd>tab|tab</kbd> or <kbd>enter|enter</kbd> to accept suggestions.

### Diagnostics

Language servers provide real-time diagnostics (errors, warnings, hints) as you code. View all diagnostics for your project using the `diagnostics: Toggle` command.
