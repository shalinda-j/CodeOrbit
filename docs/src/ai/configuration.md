# Configuration

There are various aspects about the Agent Panel that you can customize.
All of them can be seen by either visiting [the Configuring CodeOrbit page](../configuring-CodeOrbit.md#agent) or by running the `CodeOrbit: open default settings` action and searching for `"agent"`.

Alternatively, you can also visit the panel's Settings view by running the `agent: open configuration` action or going to the top-right menu and hitting "Settings".

## LLM Providers

CodeOrbit supports multiple large language model providers.
Here's an overview of the supported providers and tool call support:

| Provider                                        | Tool Use Supported                                                                                                                                                          |
| ----------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [Amazon Bedrock](#amazon-bedrock)               | Depends on the model                                                                                                                                                        |
| [Anthropic](#anthropic)                         | ✅                                                                                                                                                                          |
| [DeepSeek](#deepseek)                           | ✅                                                                                                                                                                          |
| [GitHub Copilot Chat](#github-copilot-chat)     | For some models ([link](https://github.com/codeorbit-industries/CodeOrbit/blob/9e0330ba7d848755c9734bf456c716bddf0973f3/crates/language_models/src/provider/copilot_chat.rs#L189-L198)) |
| [Google AI](#google-ai)                         | ✅                                                                                                                                                                          |
| [LM Studio](#lmstudio)                          | ✅                                                                                                                                                                          |
| [Mistral](#mistral)                             | ✅                                                                                                                                                                          |
| [Ollama](#ollama)                               | ✅                                                                                                                                                                          |
| [OpenAI](#openai)                               | ✅                                                                                                                                                                          |
| [OpenAI API Compatible](#openai-api-compatible) | 🚫                                                                                                                                                                          |
| [OpenRouter](#openrouter)                       | ✅                                                                                                                                                                          |
| [Vercel](#vercel-v0)                            | ✅                                                                                                                                                                          |
| [xAI](#xai)                                     | ✅                                                                                                                                                                          |

## Use Your Own Keys {#use-your-own-keys}

While CodeOrbit offers hosted versions of models through [our various plans](./plans-and-usage.md), we're always happy to support users wanting to supply their own API keys.
Below, you can learn how to do that for each provider.

> Using your own API keys is _free_—you do not need to subscribe to a CodeOrbit plan to use our AI features with your own keys.

### Amazon Bedrock {#amazon-bedrock}

> ✅ Supports tool use with models that support streaming tool use.
> More details can be found in the [Amazon Bedrock's Tool Use documentation](https://docs.aws.amazon.com/bedrock/latest/userguide/conversation-inference-supported-models-features.html).

To use Amazon Bedrock's models, an AWS authentication is required.
Ensure your credentials have the following permissions set up:

- `bedrock:InvokeModelWithResponseStream`
- `bedrock:InvokeModel`
- `bedrock:ConverseStream`

Your IAM policy should look similar to:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "bedrock:InvokeModel",
        "bedrock:InvokeModelWithResponseStream",
        "bedrock:ConverseStream"
      ],
      "Resource": "*"
    }
  ]
}
```

With that done, choose one of the two authentication methods:

#### Authentication via Named Profile (Recommended)

1. Ensure you have the AWS CLI installed and configured with a named profile
2. Open your `settings.json` (`CodeOrbit: open settings`) and include the `bedrock` key under `language_models` with the following settings:
   ```json
   {
     "language_models": {
       "bedrock": {
         "authentication_method": "named_profile",
         "region": "your-aws-region",
         "profile": "your-profile-name"
       }
     }
   }
   ```

#### Authentication via Static Credentials

While it's possible to configure through the Agent Panel settings UI by entering your AWS access key and secret directly, we recommend using named profiles instead for better security practices.
To do this:

1. Create an IAM User that you can assume in the [IAM Console](https://us-east-1.console.aws.amazon.com/iam/home?region=us-east-1#/users).
2. Create security credentials for that User, save them and keep them secure.
3. Open the Agent Configuration with (`agent: open configuration`) and go to the Amazon Bedrock section
4. Copy the credentials from Step 2 into the respective **Access Key ID**, **Secret Access Key**, and **Region** fields.

#### Cross-Region Inference

The CodeOrbit implementation of Amazon Bedrock uses [Cross-Region inference](https://docs.aws.amazon.com/bedrock/latest/userguide/cross-region-inference.html) for all the models and region combinations that support it.
With Cross-Region inference, you can distribute traffic across multiple AWS Regions, enabling higher throughput.

For example, if you use `Claude Sonnet 3.7 Thinking` from `us-east-1`, it may be processed across the US regions, namely: `us-east-1`, `us-east-2`, or `us-west-2`.
Cross-Region inference requests are kept within the AWS Regions that are part of the geography where the data originally resides.
For example, a request made within the US is kept within the AWS Regions in the US.

Although the data remains stored only in the source Region, your input prompts and output results might move outside of your source Region during cross-Region inference.
All data will be transmitted encrypted across Amazon's secure network.

We will support Cross-Region inference for each of the models on a best-effort basis, please refer to the [Cross-Region Inference method Code](https://github.com/codeorbit-industries/CodeOrbit/blob/main/crates/bedrock/src/models.rs#L297).

For the most up-to-date supported regions and models, refer to the [Supported Models and Regions for Cross Region inference](https://docs.aws.amazon.com/bedrock/latest/userguide/inference-profiles-support.html).

### Anthropic {#anthropic}

> ✅ Supports tool use

You can use Anthropic models by choosing it via the model dropdown in the Agent Panel.

1. Sign up for Anthropic and [create an API key](https://console.anthropic.com/settings/keys)
2. Make sure that your Anthropic account has credits
3. Open the settings view (`agent: open configuration`) and go to the Anthropic section
4. Enter your Anthropic API key

Even if you pay for Claude Pro, you will still have to [pay for additional credits](https://console.anthropic.com/settings/plans) to use it via the API.

CodeOrbit will also use the `ANTHROPIC_API_KEY` environment variable if it's defined.

#### Custom Models {#anthropic-custom-models}

You can add custom models to the Anthropic provider by adding the following to your CodeOrbit `settings.json`:

```json
{
  "language_models": {
    "anthropic": {
      "available_models": [
        {
          "name": "claude-3-5-sonnet-20240620",
          "display_name": "Sonnet 2024-June",
          "max_tokens": 128000,
          "max_output_tokens": 2560,
          "cache_configuration": {
            "max_cache_anchors": 10,
            "min_total_token": 10000,
            "should_speculate": false
          },
          "tool_override": "some-model-that-supports-toolcalling"
        }
      ]
    }
  }
}
```

Custom models will be listed in the model dropdown in the Agent Panel.

You can configure a model to use [extended thinking](https://docs.anthropic.com/en/docs/about-claude/models/extended-thinking-models) (if it supports it) by changing the mode in your model's configuration to `thinking`, for example:

```json
{
  "name": "claude-sonnet-4-latest",
  "display_name": "claude-sonnet-4-thinking",
  "max_tokens": 200000,
  "mode": {
    "type": "thinking",
    "budget_tokens": 4_096
  }
}
```

### DeepSeek {#deepseek}

> ✅ Supports tool use

1. Visit the DeepSeek platform and [create an API key](https://platform.deepseek.com/api_keys)
2. Open the settings view (`agent: open configuration`) and go to the DeepSeek section
3. Enter your DeepSeek API key

The DeepSeek API key will be saved in your keychain.

CodeOrbit will also use the `DEEPSEEK_API_KEY` environment variable if it's defined.

#### Custom Models {#deepseek-custom-models}

The CodeOrbit agent comes pre-configured to use the latest version for common models (DeepSeek Chat, DeepSeek Reasoner).
If you wish to use alternate models or customize the API endpoint, you can do so by adding the following to your CodeOrbit `settings.json`:

```json
{
  "language_models": {
    "deepseek": {
      "api_url": "https://api.deepseek.com",
      "available_models": [
        {
          "name": "deepseek-chat",
          "display_name": "DeepSeek Chat",
          "max_tokens": 64000
        },
        {
          "name": "deepseek-reasoner",
          "display_name": "DeepSeek Reasoner",
          "max_tokens": 64000,
          "max_output_tokens": 4096
        }
      ]
    }
  }
}
```

Custom models will be listed in the model dropdown in the Agent Panel.
You can also modify the `api_url` to use a custom endpoint if needed.

### GitHub Copilot Chat {#github-copilot-chat}

> ✅ Supports tool use in some cases.
> Visit [the Copilot Chat code](https://github.com/codeorbit-industries/CodeOrbit/blob/9e0330ba7d848755c9734bf456c716bddf0973f3/crates/language_models/src/provider/copilot_chat.rs#L189-L198) for the supported subset.

You can use GitHub Copilot Chat with the CodeOrbit agent by choosing it via the model dropdown in the Agent Panel.

1. Open the settings view (`agent: open configuration`) and go to the GitHub Copilot Chat section
2. Click on `Sign in to use GitHub Copilot`, follow the steps shown in the modal.

Alternatively, you can provide an OAuth token via the `GH_COPILOT_TOKEN` environment variable.

> **Note**: If you don't see specific models in the dropdown, you may need to enable them in your [GitHub Copilot settings](https://github.com/settings/copilot/features).

To use Copilot Enterprise with CodeOrbit (for both agent and inline completions), you must configure your enterprise endpoint as described in [Configuring GitHub Copilot Enterprise](./edit-prediction.md#github-copilot-enterprise).

### Google AI {#google-ai}

> ✅ Supports tool use

You can use Gemini models with the CodeOrbit agent by choosing it via the model dropdown in the Agent Panel.

1. Go to the Google AI Studio site and [create an API key](https://aistudio.google.com/app/apikey).
2. Open the settings view (`agent: open configuration`) and go to the Google AI section
3. Enter your Google AI API key and press enter.

The Google AI API key will be saved in your keychain.

CodeOrbit will also use the `GEMINI_API_KEY` environment variable if it's defined. See [Using Gemini API keys](Using Gemini API keys) in the Gemini docs for more.

#### Custom Models {#google-ai-custom-models}

By default, CodeOrbit will use `stable` versions of models, but you can use specific versions of models, including [experimental models](https://ai.google.dev/gemini-api/docs/models/experimental-models). You can configure a model to use [thinking mode](https://ai.google.dev/gemini-api/docs/thinking) (if it supports it) by adding a `mode` configuration to your model. This is useful for controlling reasoning token usage and response speed. If not specified, Gemini will automatically choose the thinking budget.

Here is an example of a custom Google AI model you could add to your CodeOrbit `settings.json`:

```json
{
  "language_models": {
    "google": {
      "available_models": [
        {
          "name": "gemini-2.5-flash-preview-05-20",
          "display_name": "Gemini 2.5 Flash (Thinking)",
          "max_tokens": 1000000,
          "mode": {
            "type": "thinking",
            "budget_tokens": 24000
          }
        }
      ]
    }
  }
}
```

Custom models will be listed in the model dropdown in the Agent Panel.

### LM Studio {#lmstudio}

> ✅ Supports tool use

1. Download and install [the latest version of LM Studio](https://lmstudio.ai/download)
2. In the app press `cmd/ctrl-shift-m` and download at least one model (e.g., qwen2.5-coder-7b). Alternatively, you can get models via the LM Studio CLI:

   ```sh
   lms get qwen2.5-coder-7b
   ```

3. Make sure the LM Studio API server is running by executing:

   ```sh
   lms server start
   ```

Tip: Set [LM Studio as a login item](https://lmstudio.ai/docs/advanced/headless#run-the-llm-service-on-machine-login) to automate running the LM Studio server.

### Mistral {#mistral}

> ✅ Supports tool use

1. Visit the Mistral platform and [create an API key](https://console.mistral.ai/api-keys/)
2. Open the configuration view (`agent: open configuration`) and navigate to the Mistral section
3. Enter your Mistral API key

The Mistral API key will be saved in your keychain.

CodeOrbit will also use the `MISTRAL_API_KEY` environment variable if it's defined.

#### Custom Models {#mistral-custom-models}

The CodeOrbit agent comes pre-configured with several Mistral models (codestral-latest, mistral-large-latest, mistral-medium-latest, mistral-small-latest, open-mistral-nemo, and open-codestral-mamba).
All the default models support tool use.
If you wish to use alternate models or customize their parameters, you can do so by adding the following to your CodeOrbit `settings.json`:

```json
{
  "language_models": {
    "mistral": {
      "api_url": "https://api.mistral.ai/v1",
      "available_models": [
        {
          "name": "mistral-tiny-latest",
          "display_name": "Mistral Tiny",
          "max_tokens": 32000,
          "max_output_tokens": 4096,
          "max_completion_tokens": 1024,
          "supports_tools": true,
          "supports_images": false
        }
      ]
    }
  }
}
```

Custom models will be listed in the model dropdown in the Agent Panel.

### Ollama {#ollama}

> ✅ Supports tool use

Download and install Ollama from [ollama.com/download](https://ollama.com/download) (Linux or macOS) and ensure it's running with `ollama --version`.

1. Download one of the [available models](https://ollama.com/models), for example, for `mistral`:

   ```sh
   ollama pull mistral
   ```

2. Make sure that the Ollama server is running. You can start it either via running Ollama.app (macOS) or launching:

   ```sh
   ollama serve
   ```

3. In the Agent Panel, select one of the Ollama models using the model dropdown.

#### Ollama Context Length {#ollama-context}

CodeOrbit has pre-configured maximum context lengths (`max_tokens`) to match the capabilities of common models.
CodeOrbit API requests to Ollama include this as the `num_ctx` parameter, but the default values do not exceed `16384` so users with ~16GB of RAM are able to use most models out of the box.

See [get_max_tokens in ollama.rs](https://github.com/codeorbit-industries/CodeOrbit/blob/main/crates/ollama/src/ollama.rs) for a complete set of defaults.

> **Note**: Token counts displayed in the Agent Panel are only estimates and will differ from the model's native tokenizer.

Depending on your hardware or use-case you may wish to limit or increase the context length for a specific model via settings.json:

```json
{
  "language_models": {
    "ollama": {
      "api_url": "http://localhost:11434",
      "available_models": [
        {
          "name": "qwen2.5-coder",
          "display_name": "qwen 2.5 coder 32K",
          "max_tokens": 32768,
          "supports_tools": true,
          "supports_thinking": true,
          "supports_images": true
        }
      ]
    }
  }
}
```

If you specify a context length that is too large for your hardware, Ollama will log an error.
You can watch these logs by running: `tail -f ~/.ollama/logs/ollama.log` (macOS) or `journalctl -u ollama -f` (Linux).
Depending on the memory available on your machine, you may need to adjust the context length to a smaller value.

You may also optionally specify a value for `keep_alive` for each available model.
This can be an integer (seconds) or alternatively a string duration like "5m", "10m", "1h", "1d", etc.
For example, `"keep_alive": "120s"` will allow the remote server to unload the model (freeing up GPU VRAM) after 120 seconds.

The `supports_tools` option controls whether the model will use additional tools.
If the model is tagged with `tools` in the Ollama catalog, this option should be supplied, and the built-in profiles `Ask` and `Write` can be used.
If the model is not tagged with `tools` in the Ollama catalog, this option can still be supplied with the value `true`; however, be aware that only the `Minimal` built-in profile will work.

The `supports_thinking` option controls whether the model will perform an explicit "thinking" (reasoning) pass before producing its final answer.
If the model is tagged with `thinking` in the Ollama catalog, set this option and you can use it in CodeOrbit.

The `supports_images` option enables the model's vision capabilities, allowing it to process images included in the conversation context.
If the model is tagged with `vision` in the Ollama catalog, set this option and you can use it in CodeOrbit.

### OpenAI {#openai}

> ✅ Supports tool use

1. Visit the OpenAI platform and [create an API key](https://platform.openai.com/account/api-keys)
2. Make sure that your OpenAI account has credits
3. Open the settings view (`agent: open configuration`) and go to the OpenAI section
4. Enter your OpenAI API key

The OpenAI API key will be saved in your keychain.

CodeOrbit will also use the `OPENAI_API_KEY` environment variable if it's defined.

#### Custom Models {#openai-custom-models}

The CodeOrbit agent comes pre-configured to use the latest version for common models (GPT-3.5 Turbo, GPT-4, GPT-4 Turbo, GPT-4o, GPT-4o mini).
To use alternate models, perhaps a preview release or a dated model release, or if you wish to control the request parameters, you can do so by adding the following to your CodeOrbit `settings.json`:

```json
{
  "language_models": {
    "openai": {
      "available_models": [
        {
          "name": "gpt-4o-2024-08-06",
          "display_name": "GPT 4o Summer 2024",
          "max_tokens": 128000
        },
        {
          "name": "o1-mini",
          "display_name": "o1-mini",
          "max_tokens": 128000,
          "max_completion_tokens": 20000
        }
      ],
      "version": "1"
    }
  }
}
```

You must provide the model's context window in the `max_tokens` parameter; this can be found in the [OpenAI model documentation](https://platform.openai.com/docs/models).

OpenAI `o1` models should set `max_completion_tokens` as well to avoid incurring high reasoning token costs.
Custom models will be listed in the model dropdown in the Agent Panel.

### OpenAI API Compatible {#openai-api-compatible}

CodeOrbit supports using OpenAI compatible APIs by specifying a custom `endpoint` and `available_models` for the OpenAI provider.

CodeOrbit supports using OpenAI compatible APIs by specifying a custom `api_url` and `available_models` for the OpenAI provider. This is useful for connecting to other hosted services (like Together AI, Anyscale, etc.) or local models.

To configure a compatible API, you can add a custom API URL for OpenAI either via the UI or by editing your `settings.json`. For example, to connect to [Together AI](https://www.together.ai/):

1.  Get an API key from your [Together AI account](https://api.together.ai/settings/api-keys).
2.  Add the following to your `settings.json`:

```json
{
  "language_models": {
    "openai": {
      "api_url": "https://api.together.xyz/v1",
      "api_key": "YOUR_TOGETHER_AI_API_KEY",
      "available_models": [
        {
          "name": "mistralai/Mixtral-8x7B-Instruct-v0.1",
          "display_name": "Together Mixtral 8x7B",
          "max_tokens": 32768,
          "supports_tools": true
        }
      ]
    }
  }
}
```

### OpenRouter {#openrouter}

> ✅ Supports tool use

OpenRouter provides access to multiple AI models through a single API. It supports tool use for compatible models.

1. Visit [OpenRouter](https://openrouter.ai) and create an account
2. Generate an API key from your [OpenRouter keys page](https://openrouter.ai/keys)
3. Open the settings view (`agent: open configuration`) and go to the OpenRouter section
4. Enter your OpenRouter API key

The OpenRouter API key will be saved in your keychain.

CodeOrbit will also use the `OPENROUTER_API_KEY` environment variable if it's defined.

#### Custom Models {#openrouter-custom-models}

You can add custom models to the OpenRouter provider by adding the following to your CodeOrbit `settings.json`:

```json
{
  "language_models": {
    "open_router": {
      "api_url": "https://openrouter.ai/api/v1",
      "available_models": [
        {
          "name": "google/gemini-2.0-flash-thinking-exp",
          "display_name": "Gemini 2.0 Flash (Thinking)",
          "max_tokens": 200000,
          "max_output_tokens": 8192,
          "supports_tools": true,
          "supports_images": true,
          "mode": {
            "type": "thinking",
            "budget_tokens": 8000
          }
        }
      ]
    }
  }
}
```

The available configuration options for each model are:

- `name` (required): The model identifier used by OpenRouter
- `display_name` (optional): A human-readable name shown in the UI
- `max_tokens` (required): The model's context window size
- `max_output_tokens` (optional): Maximum tokens the model can generate
- `max_completion_tokens` (optional): Maximum completion tokens
- `supports_tools` (optional): Whether the model supports tool/function calling
- `supports_images` (optional): Whether the model supports image inputs
- `mode` (optional): Special mode configuration for thinking models

You can find available models and their specifications on the [OpenRouter models page](https://openrouter.ai/models).

Custom models will be listed in the model dropdown in the Agent Panel.

### Vercel v0 {#vercel-v0}

> ✅ Supports tool use

[Vercel v0](https://vercel.com/docs/v0/api) is an expert model for generating full-stack apps, with framework-aware completions optimiCodeOrbit for modern stacks like Next.js and Vercel.
It supports text and image inputs and provides fast streaming responses.

The v0 models are [OpenAI-compatible models](/#openai-api-compatible), but Vercel is listed as first-class provider in the panel's settings view.

To start using it with CodeOrbit, ensure you have first created a [v0 API key](https://v0.dev/chat/settings/keys).
Once you have it, paste it directly into the Vercel provider section in the panel's settings view.

You should then find it as `v0-1.5-md` in the model dropdown in the Agent Panel.

### xAI {#xai}

> ✅ Supports tool use

CodeOrbit has first-class support for [xAI](https://x.ai/) models. You can use your own API key to access Grok models.

1. [Create an API key in the xAI Console](https://console.x.ai/team/default/api-keys)
2. Open the settings view (`agent: open configuration`) and go to the **xAI** section
3. Enter your xAI API key

The xAI API key will be saved in your keychain. CodeOrbit will also use the `XAI_API_KEY` environment variable if it's defined.

> **Note:** While the xAI API is OpenAI-compatible, CodeOrbit has first-class support for it as a dedicated provider. For the best experience, we recommend using the dedicated `x_ai` provider configuration instead of the [OpenAI API Compatible](#openai-api-compatible) method.

#### Custom Models {#xai-custom-models}

The CodeOrbit agent comes pre-configured with common Grok models. If you wish to use alternate models or customize their parameters, you can do so by adding the following to your CodeOrbit `settings.json`:

```json
{
  "language_models": {
    "x_ai": {
      "api_url": "https://api.x.ai/v1",
      "available_models": [
        {
          "name": "grok-1.5",
          "display_name": "Grok 1.5",
          "max_tokens": 131072,
          "max_output_tokens": 8192
        },
        {
          "name": "grok-1.5v",
          "display_name": "Grok 1.5V (Vision)",
          "max_tokens": 131072,
          "max_output_tokens": 8192,
          "supports_images": true
        }
      ]
    }
  }
}
```

## Advanced Configuration {#advanced-configuration}

### Custom Provider Endpoints {#custom-provider-endpoint}

You can use a custom API endpoint for different providers, as long as it's compatible with the provider's API structure.
To do so, add the following to your `settings.json`:

```json
{
  "language_models": {
    "some-provider": {
      "api_url": "http://localhost:11434"
    }
  }
}
```

Where `some-provider` can be any of the following values: `anthropic`, `google`, `ollama`, `openai`.

### Default Model {#default-model}

CodeOrbit's hosted LLM service sets `claude-sonnet-4` as the default model.
However, you can change it either via the model dropdown in the Agent Panel's bottom-right corner or by manually editing the `default_model` object in your settings:

```json
{
  "agent": {
    "version": "2",
    "default_model": {
      "provider": "codeorbit.dev",
      "model": "gpt-4o"
    }
  }
}
```

### Feature-specific Models {#feature-specific-models}

If a feature-specific model is not set, it will fall back to using the default model, which is the one you set on the Agent Panel.

You can configure the following feature-specific models:

- Thread summary model: Used for generating thread summaries
- Inline assistant model: Used for the inline assistant feature
- Commit message model: Used for generating Git commit messages

Example configuration:

```json
{
  "agent": {
    "version": "2",
    "default_model": {
      "provider": "codeorbit.dev",
      "model": "claude-sonnet-4"
    },
    "inline_assistant_model": {
      "provider": "anthropic",
      "model": "claude-3-5-sonnet"
    },
    "commit_message_model": {
      "provider": "openai",
      "model": "gpt-4o-mini"
    },
    "thread_summary_model": {
      "provider": "google",
      "model": "gemini-2.0-flash"
    }
  }
}
```

### Alternative Models for Inline Assists {#alternative-assists}

You can configure additional models that will be used to perform inline assists in parallel.
When you do this, the inline assist UI will surface controls to cycle between the alternatives generated by each model.

The models you specify here are always used in _addition_ to your [default model](#default-model).
For example, the following configuration will generate two outputs for every assist.
One with Claude 3.7 Sonnet, and one with GPT-4o.

```json
{
  "agent": {
    "default_model": {
      "provider": "codeorbit.dev",
      "model": "claude-sonnet-4"
    },
    "inline_alternatives": [
      {
        "provider": "codeorbit.dev",
        "model": "gpt-4o"
      }
    ],
    "version": "2"
  }
}
```

### Default View

Use the `default_view` setting to set change the default view of the Agent Panel.
You can choose between `thread` (the default) and `text_thread`:

```json
{
  "agent": {
    "default_view": "text_thread"
  }
}
```

### Edit Card

Use the `expand_edit_card` setting to control whether edit cards show the full diff in the Agent Panel.
It is set to `true` by default, but if set to false, the card's height is capped to a certain number of lines, requiring a click to be expanded.

```json
{
  "agent": {
    "expand_edit_card": "false"
  }
}
```

This setting is currently only available in Preview.
It should be up in Stable by the next release.

### Terminal Card

Use the `expand_terminal_card` setting to control whether terminal cards show the command output in the Agent Panel.
It is set to `true` by default, but if set to false, the card will be fully collapsed even while the command is running, requiring a click to be expanded.

```json
{
  "agent": {
    "expand_terminal_card": "false"
  }
}
```

This setting is currently only available in Preview.
It should be up in Stable by the next release.
