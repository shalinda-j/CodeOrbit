# Model Temperature

CodeOrbit's settings allow you to specify a custom temperature for a provider and/or model:

```json
"model_parameters": [
      // To set parameters for all requests to OpenAI models:
      {
        "provider": "openai",
        "temperature": 0.5
      },
      // To set parameters for all requests in general:
      {
        "temperature": 0
      },
      // To set parameters for a specific provider and model:
      {
        "provider": "codeorbit.dev",
        "model": "claude-sonnet-4",
        "temperature": 1.0
      }
    ],
```
