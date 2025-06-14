﻿# Building CodeOrbit for Linux

## Repository

Clone down the [CodeOrbit repository](https://github.com/CodeOrbit-industries/CodeOrbit).

## Dependencies

- Install [rustup](https://www.rust-lang.org/tools/install)

- Install the necessary system libraries:

  ```sh
  script/linux
  ```

  If you prefer to install the system libraries manually, you can find the list of required packages in the `script/linux` file.

## Backend dependencies

> This section is still in development. The instructions are not yet complete.

If you are developing collaborative features of CodeOrbit, you'll need to install the dependencies of CodeOrbit's `collab` server:

- Install [Postgres](https://www.postgresql.org/download/linux/)
- Install [Livekit](https://github.com/livekit/livekit-cli) and [Foreman](https://theforeman.org/manuals/3.9/quickstart_guide.html)

Alternatively, if you have [Docker](https://www.docker.com/) installed you can bring up all the `collab` dependencies using Docker Compose:

```sh
docker compose up -d
```

## Building from source

Once the dependencies are installed, you can build CodeOrbit using [Cargo](https://doc.rust-lang.org/cargo/).

For a debug build of the editor:

```sh
cargo run
```

And to run the tests:

```sh
cargo test --workspace
```

In release mode, the primary user interface is the `cli` crate. You can run it in development with:

```sh
cargo run -p cli
```

## Installing a development build

You can install a local build on your machine with:

```sh
./script/install-linux
```

This will build CodeOrbit and the cli in release mode and make them available at `~/.local/bin/CodeOrbit`, installing .desktop files to `~/.local/share`.

> **_Note_**: If you encounter linker errors similar to the following:
>
> ```bash
> error: linking with `cc` failed: exit status: 1 ...
> = note: /usr/bin/ld: /tmp/rustcISMaod/libaws_lc_sys-79f08eb6d32e546e.rlib(f8e4fd781484bd36-bcm.o): in function `aws_lc_0_25_0_handle_cpu_env':
>           /aws-lc/crypto/fipsmodule/cpucap/cpu_intel.c:(.text.aws_lc_0_25_0_handle_cpu_env+0x63): undefined reference to `__isoc23_sscanf'
>           /usr/bin/ld: /tmp/rustcISMaod/libaws_lc_sys-79f08eb6d32e546e.rlib(f8e4fd781484bd36-bcm.o): in function `pkey_rsa_ctrl_str':
>           /aws-lc/crypto/fipsmodule/evp/p_rsa.c:741:(.text.pkey_rsa_ctrl_str+0x20d): undefined reference to `__isoc23_strtol'
>           /usr/bin/ld: /aws-lc/crypto/fipsmodule/evp/p_rsa.c:752:(.text.pkey_rsa_ctrl_str+0x258): undefined reference to `__isoc23_strtol'
>           collect2: error: ld returned 1 exit status
>   = note: some `extern` functions couldn't be found; some native libraries may need to be installed or have their path specified
>   = note: use the `-l` flag to specify native libraries to link
>   = note: use the `cargo:rustc-link-lib` directive to specify the native libraries to link with Cargo (see https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-link-lib)
> error: could not compile `remote_server` (bin "remote_server") due to 1 previous error
> ```
>
> **Cause**:
> this is caused by known bugs in aws-lc-rs(doesn't support GCC >= 14): [FIPS fails to build with GCC >= 14](https://github.com/aws/aws-lc-rs/issues/569)
> & [GCC-14 - build failure for FIPS module](https://github.com/aws/aws-lc/issues/2010)
>
> You can refer to [linux: Linker error for remote_server when using script/install-linux](https://github.com/CodeOrbit-industries/CodeOrbit/issues/24880) for more information.
>
> **Workarounds**:
> Set the remote server target to `x86_64-unknown-linux-gnu` like so `export REMOTE_SERVER_TARGET=x86_64-unknown-linux-gnu; script/install-linux`

## Wayland & X11

CodeOrbit supports both X11 and Wayland. By default, we pick whichever we can find at runtime. If you're on Wayland and want to run in X11 mode, use the environment variable `WAYLAND_DISPLAY=''`.

## Notes for packaging CodeOrbit

Thank you for taking on the task of packaging CodeOrbit!

### Technical requirements

CodeOrbit has two main binaries:

- You will need to build `crates/cli` and make its binary available in `$PATH` with the name `CodeOrbit`.
- You will need to build `crates/CodeOrbit` and put it at `$PATH/to/cli/../../libexec/CodeOrbit-editor`. For example, if you are going to put the cli at `~/.local/bin/CodeOrbit` put CodeOrbit at `~/.local/libexec/CodeOrbit-editor`. As some linux distributions (notably Arch) discourage the use of `libexec`, you can also put this binary at `$PATH/to/cli/../../lib/CodeOrbit/CodeOrbit-editor` (e.g. `~/.local/lib/CodeOrbit/CodeOrbit-editor`) instead.
- If you are going to provide a `.desktop` file you can find a template in `crates/CodeOrbit/resources/CodeOrbit.desktop.in`, and use `envsubst` to populate it with the values required. This file should also be renamed to `$APP_ID.desktop` so that the file [follows the FreeDesktop standards](https://github.com/CodeOrbit-industries/CodeOrbit/issues/12707#issuecomment-2168742761).
- You will need to ensure that the necessary libraries are installed. You can get the current list by [inspecting the built binary](https://github.com/CodeOrbit-industries/CodeOrbit/blob/935cf542aebf55122ce6ed1c91d0fe8711970c82/script/bundle-linux#L65-L67) on your system.
- For an example of a complete build script, see [script/bundle-linux](https://github.com/CodeOrbit-industries/CodeOrbit/blob/935cf542aebf55122ce6ed1c91d0fe8711970c82/script/bundle-linux).
- You can disable CodeOrbit's auto updates and provide instructions for users who try to update CodeOrbit manually by building (or running) CodeOrbit with the environment variable `codeorbit_UPDATE_EXPLANATION`. For example: `codeorbit_UPDATE_EXPLANATION="Please use flatpak to update CodeOrbit."`.
- Make sure to update the contents of the `crates/CodeOrbit/RELEASE_CHANNEL` file to 'nightly', 'preview', or 'stable', with no newline. This will cause CodeOrbit to use the credentials manager to remember a user's login.

### Other things to note

At CodeOrbit, our priority has been to move fast and bring the latest technology to our users. We've long been frustrated at having software that is slow, out of date, or hard to configure, and so we've built our editor to those tastes.

However, we realize that many distros have other priorities. We want to work with everyone to bring CodeOrbit to their favorite platforms. But there is a long way to go:

- CodeOrbit is a fast-moving early-phase project. We typically release 2-3 builds per week to fix user-reported issues and release major features.
- There are a couple of other `CodeOrbit` binaries that may be present on Linux systems ([1](https://openzfs.github.io/openzfs-docs/man/v2.2/8/CodeOrbit.8.html), [2](https://CodeOrbit.brimdata.io/docs/commands/CodeOrbit)). If you want to rename our CLI binary because of these issues, we suggest `codeorbit-edit`, `codeorbit-editor`, or `CodeOrbit-cli`.
- CodeOrbit automatically installs the correct version of common developer tools in the same way as rustup/rbenv/pyenv, etc. We understand this is contentious, [see here](https://github.com/CodeOrbit-industries/CodeOrbit/issues/12589).
- We allow users to install extensions locally and from [CodeOrbit-industries/extensions](https://github.com/CodeOrbit-industries/extensions). These extensions may install further tooling as needed, such as language servers. In the long run, we would like to make this safer, [see here](https://github.com/CodeOrbit-industries/CodeOrbit/issues/12358).
- CodeOrbit connects to several online services by default (AI, telemetry, collaboration). AI and our telemetry can be disabled by your users with their CodeOrbit settings or by patching our [default settings file](https://github.com/CodeOrbit-industries/CodeOrbit/blob/main/assets/settings/default.json).
- As a result of the above issues, CodeOrbit currently does not play nice with sandboxes, [see here](https://github.com/CodeOrbit-industries/CodeOrbit/pull/12006#issuecomment-2130421220)

## Flatpak

> CodeOrbit's current Flatpak integration exits the sandbox on startup. Workflows that rely on Flatpak's sandboxing may not work as expected.

To build & install the Flatpak package locally follow the steps below:

1. Install Flatpak for your distribution as outlined [here](https://flathub.org/setup).
2. Run the `script/flatpak/deps` script to install the required dependencies.
3. Run `script/flatpak/bundle-flatpak`.
4. Now the package has been installed and has a bundle available at `target/release/{app-id}.flatpak`.

## Memory profiling

[`heaptrack`](https://github.com/KDE/heaptrack) is quite useful for diagnosing memory leaks. To install it:

```sh
$ sudo apt install heaptrack heaptrack-gui
$ cargo install cargo-heaptrack
```

Then, to build and run CodeOrbit with the profiler attached:

```sh
$ cargo heaptrack -b CodeOrbit
```

When this CodeOrbit instance is exited, terminal output will include a command to run `heaptrack_interpret` to convert the `*.raw.zst` profile to a `*.zst` file which can be passed to `heaptrack_gui` for viewing.

## Troubleshooting

### Cargo errors claiming that a dependency is using unstable features

Try `cargo clean` and `cargo build`.

### Vulkan/GPU issues

If CodeOrbit crashes at runtime due to GPU or vulkan issues, you can try running [vkcube](https://github.com/krh/vkcube) (usually available as part of the `vulkaninfo` package on various distributions) to try to troubleshoot where the issue is coming from. Try running in both X11 and wayland modes by running `vkcube -m [x11|wayland]`. Some versions of `vkcube` use `vkcube` to run in X11 and `vkcube-wayland` to run in wayland.

If you have multiple GPUs, you can also try running CodeOrbit on a different one to figure out where the issue comes from. You can do so a couple different ways:
Option A: with [vkdevicechooser](https://github.com/jiriks74/vkdevicechooser))
Or Option B: By using the `codeorbit_DEVICE_ID={device_id}` environment variable to specify the device ID.

You can obtain the device ID of your GPU by running `lspci -nn | grep VGA` which will output each GPU on one line like:

```
08:00.0 VGA compatible controller [0300]: NVIDIA Corporation GA104 [GeForce RTX 3070] [10de:2484] (rev a1)
```

where the device ID here is `2484`. This value is in hexadecimal, so to force CodeOrbit to use this specific GPU you would set the environment variable like so:

```
codeorbit_DEVICE_ID=0x2484
```

Make sure to export the variable if you choose to define it globally in a `.bashrc` or similar

#### Reporting Vulkan/GPU issues

When reporting issues where CodeOrbit fails to start due to graphics initialization errors on GitHub, it can be impossible to run the `CodeOrbit: copy system specs into clipboard` command like we instruct you to in our issue template. We provide an alternative way to collect the system specs specifically for this situation.

Passing the `--system-specs` flag to CodeOrbit like

```sh
CodeOrbit --system-specs
```

will print the system specs to the terminal like so. It is strongly recommended to copy the output verbatim into the issue on GitHub, as it uses markdown formatting to ensure the output is readable.

Additionally, it is extremely beneficial to provide the contents of your CodeOrbit log when reporting such issues. The log is usually stored at `~/.local/share/CodeOrbit/logs/CodeOrbit.log`. The recommended process for producing a helpful log file is as follows:

```sh
truncate -s 0 ~/.local/share/CodeOrbit/logs/CodeOrbit.log # Clear the log file
codeorbit_LOG=blade_graphics=info CodeOrbit .
cat ~/.local/share/CodeOrbit/logs/CodeOrbit.log
# copy the output
```

Or, if you have the CodeOrbit cli setup, you can do

```sh
codeorbit_LOG=blade_graphics=info /path/to/CodeOrbit/cli --foreground .
# copy the output
```

It is also highly recommended when pasting the log into a github issue, to do so with the following template:

> **_Note_**: The whitespace in the template is important, and will cause incorrect formatting if not preserved.

````
<details><summary>CodeOrbit Log</summary>

```
{CodeOrbit log contents}
```

</details>
````

This will cause the logs to be collapsed by default, making it easier to read the issue.
