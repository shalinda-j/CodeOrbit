# Deno

Deno support is available through the [Deno extension](https://github.com/CodeOrbit-extensions/deno).

- Language server: [Deno Language Server](https://docs.deno.com/runtime/manual/advanced/language_server/overview/)

## Deno Configuration

To use the Deno Language Server with TypeScript and TSX files, you will likely wish to disable the default language servers and enable deno by adding the following to your settings.json:

```json
{
  "lsp": {
    "deno": {
      "settings": {
        "deno": {
          "enable": true
        }
      }
    }
  },
  "languages": {
    "JavaScript": {
      "language_servers": [
        "deno",
        "!typescript-language-server",
        "!vtsls",
        "!eslint"
      ],
      "formatter": "language_server"
    },
    "TypeScript": {
      "language_servers": [
        "deno",
        "!typescript-language-server",
        "!vtsls",
        "!eslint"
      ],
      "formatter": "language_server"
    },
    "TSX": {
      "language_servers": [
        "deno",
        "!typescript-language-server",
        "!vtsls",
        "!eslint"
      ],
      "formatter": "language_server"
    }
  }
}
```

See [Configuring supported languages](../configuring-languages.md) in the CodeOrbit documentation for more information.

<!--
TBD: Deno Typescript REPL instructions [docs/repl#typescript-deno](../repl.md#typescript-deno)
-->

## See also:

- [TypeScript](./typescript.md)
- [JavaScript](./javascript.md)
