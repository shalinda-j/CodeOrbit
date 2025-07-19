# CodeOrbit on Linux

## Standard Installation

For most people we recommend using the script on the [download](https://codeorbit.dev/download) page to install CodeOrbit:

```sh
curl -f https://codeorbit.dev/install.sh | sh
```

We also offer a preview build of CodeOrbit which receives updates about a week ahead of stable. You can install it with:

```sh
curl -f https://codeorbit.dev/install.sh | CODEORBIT_CHANNEL=preview sh
```

The CodeOrbit installed by the script works best on systems that:

- have a Vulkan compatible GPU available (for example Linux on an M-series macBook)
- have a system-wide glibc (NixOS and Alpine do not by default)
  - x86_64 (Intel/AMD): glibc version >= 2.31 (Ubuntu 20 and newer)
  - aarch64 (ARM): glibc version >= 2.35 (Ubuntu 22 and newer)

Both Nix and Alpine have third-party CodeOrbit packages available (though they are currently a few weeks out of date). If you'd like to use our builds they do work if you install a glibc compatibility layer. On NixOS you can try [nix-ld](https://github.com/Mic92/nix-ld), and on Alpine [gcompat](https://wiki.alpinelinux.org/wiki/Running_glibc_programs).

You will need to build from source for:

- architectures other than 64-bit Intel or 64-bit ARM (for example a 32-bit or RISC-V machine)
- Redhat Enterprise Linux 8.x, Rocky Linux 8, AlmaLinux 8, Amazon Linux 2 on all architectures
- Redhat Enterprise Linux 9.x, Rocky Linux 9.3, AlmaLinux 8, Amazon Linux 2023 on aarch64 (x86_x64 OK)

## Other ways to install CodeOrbit on Linux

CodeOrbit is open source, and [you can install from source](./development/linux.md).

### Installing via a package manager

There are several third-party CodeOrbit packages for various Linux distributions and package managers, sometimes under `CodeOrbit-editor`. You may be able to install CodeOrbit using these packages:

- Flathub: [`dev.CodeOrbit.CodeOrbit`](https://flathub.org/apps/dev.CodeOrbit.CodeOrbit)
- Arch: [`CodeOrbit`](https://archlinux.org/packages/extra/x86_64/CodeOrbit/)
- Arch (AUR): [`CodeOrbit-git`](https://aur.archlinux.org/packages/CodeOrbit-git), [`CodeOrbit-preview`](https://aur.archlinux.org/packages/CodeOrbit-preview), [`CodeOrbit-preview-bin`](https://aur.archlinux.org/packages/CodeOrbit-preview-bin)
- Alpine: `CodeOrbit` ([aarch64](https://pkgs.alpinelinux.org/package/edge/testing/aarch64/CodeOrbit)) ([x86_64](https://pkgs.alpinelinux.org/package/edge/testing/x86_64/CodeOrbit))
- Nix: `CodeOrbit-editor` ([unstable](https://search.nixos.org/packages?channel=unstable&show=CodeOrbit-editor))
- Fedora/Ultramarine (Terra): [`CodeOrbit`](https://github.com/terrapkg/packages/tree/frawhide/anda/devs/CodeOrbit/stable), [`CodeOrbit-preview`](https://github.com/terrapkg/packages/tree/frawhide/anda/devs/CodeOrbit/preview), [`CodeOrbit-nightly`](https://github.com/terrapkg/packages/tree/frawhide/anda/devs/CodeOrbit/nightly)
- Solus: [`CodeOrbit`](https://github.com/getsolus/packages/tree/main/packages/z/CodeOrbit)
- Parabola: [`CodeOrbit`](https://www.parabola.nu/packages/extra/x86_64/CodeOrbit/)
- Manjaro: [`CodeOrbit`](https://packages.manjaro.org/?query=CodeOrbit)

See [Repology](https://repology.org/project/codeorbit-editor/versions) for a list of CodeOrbit packages in various repositories.

When installing a third-party package please be aware that it may not be completely up to date and may be slightly different from the CodeOrbit we package (a common change is to rename the binary to `codeedit` or `codeeditor` to avoid conflicting with other packages).

We'd love your help making CodeOrbit available for everyone. If CodeOrbit is not yet available for your package manager, and you would like to fix that, we have some notes on [how to do it](./development/linux.md#notes-for-packaging-codeorbit).

### Downloading manually

If you'd prefer, you can install CodeOrbit by downloading our pre-built .tar.gz. This is the same artifact that our install script uses, but you can customize the location of your installation by modifying the instructions below:

Download the `.tar.gz` file:

- [codeorbit-linux-x86_64.tar.gz](https://codeorbit.dev/api/releases/stable/latest/codeorbit-linux-x86_64.tar.gz) ([preview](https://codeorbit.dev/api/releases/preview/latest/codeorbit-linux-x86_64.tar.gz))
- [codeorbit-linux-aarch64.tar.gz](https://codeorbit.dev/api/releases/stable/latest/codeorbit-linux-aarch64.tar.gz)
  ([preview](https://codeorbit.dev/api/releases/preview/latest/codeorbit-linux-aarch64.tar.gz))

Then ensure that the `codeorbit` binary in the tarball is on your path. The easiest way is to unpack the tarball and create a symlink:

```sh
mkdir -p ~/.local
# extract codeorbit to ~/.local/codeorbit.app/
tar -xvf <path/to/download>.tar.gz -C ~/.local
# link the codeorbit binary to ~/.local/bin (or another directory in your $PATH)
ln -sf ~/.local/codeorbit.app/bin/codeorbit ~/.local/bin/codeorbit
```

If you'd like integration with an XDG-compatible desktop environment, you will also need to install the `.desktop` file:

```sh
cp ~/.local/codeorbit.app/share/applications/codeorbit.desktop ~/.local/share/applications/dev.codeorbit.CodeOrbit.desktop
sed -i "s|Icon=codeorbit|Icon=$HOME/.local/codeorbit.app/share/icons/hicolor/512x512/apps/codeorbit.png|g" ~/.local/share/applications/dev.codeorbit.CodeOrbit.desktop
sed -i "s|Exec=codeorbit|Exec=$HOME/.local/codeorbit.app/libexec/codeorbit-editor|g" ~/.local/share/applications/dev.codeorbit.CodeOrbit.desktop
```

## Uninstalling CodeOrbit

### Standard Uninstall

If CodeOrbit was installed using the default installation script, it can be uninstalled by supplying the `--uninstall` flag to the `codeorbit` shell command

```sh
codeorbit --uninstall
```

If there are no errors, the shell will then prompt you whether you'd like to keep your preferences or delete them. After making a choice, you should see a message that CodeOrbit was successfully uninstalled.

In the case that the `codeorbit` shell command was not found in your PATH, you can try one of the following commands

```sh
$HOME/.local/bin/codeorbit --uninstall
```

or

```sh
$HOME/.local/codeorbit.app/bin.codeorbit --uninstall
```

The first case might fail if a symlink was not properly established between `$HOME/.local/bin/codeorbit` and `$HOME/.local/codeorbit.app/bin.codeorbit`. But the second case should work as long as CodeOrbit was installed to its default location.

If CodeOrbit was installed to a different location, you must invoke the `codeorbit` binary stored in that installation directory and pass the `--uninstall` flag to it in the same format as the previous commands.

### Package Manager

If CodeOrbit was installed using a package manager, please consult the documentation for that package manager on how to uninstall a package.

## Troubleshooting

Linux works on a large variety of systems configured in many different ways. We primarily test CodeOrbit on a vanilla Ubuntu setup, as it is the most common distribution our users use, that said we do expect it to work on a wide variety of machines.

### CodeOrbit fails to start

If you see an error like "/lib64/libc.so.6: version 'GLIBC_2.29' not found" it means that your distribution's version of glibc is too old. You can either upgrade your system, or [install CodeOrbit from source](./development/linux.md).

### Graphics issues

#### CodeOrbit fails to open windows

CodeOrbit requires a GPU to run effectively. Under the hood, we use [Vulkan](https://www.vulkan.org/) to communicate with your GPU. If you are seeing problems with performance, or CodeOrbit fails to load, it is possible that Vulkan is the culprit.

If you see a notification saying `CodeOrbit failed to open a window: NoSupportedDeviceFound` this means that Vulkan cannot find a compatible GPU. you can try running [vkcube](https://github.com/krh/vkcube) (usually available as part of the `vulkaninfo` or `vulkan-tools` package on various distributions) to try to troubleshoot where the issue is coming from like so:

```
vkcube
```

> **_Note_**: Try running in both X11 and wayland modes by running `vkcube -m [x11|wayland]`. Some versions of `vkcube` use `vkcube` to run in X11 and `vkcube-wayland` to run in wayland.

This should output a line describing your current graphics setup and show a rotating cube. If this does not work, you should be able to fix it by installing Vulkan compatible GPU drivers, however in some cases (for example running Linux on an Arm-based MacBook) there is no Vulkan support yet.

You can find out which graphics card CodeOrbit is using by looking in the CodeOrbit log (`~/.local/share/CodeOrbit/logs/CodeOrbit.log`) for `Using GPU: ...`.

If you see errors like `ERROR_INITIALIZATION_FAILED` or `GPU Crashed` or `ERROR_SURFACE_LOST_KHR` then you may be able to work around this by installing different drivers for your GPU, or by selecting a different GPU to run on. (See [#14225](https://github.com/codeorbit-industries/CodeOrbit/issues/14225))

On some systems the file `/etc/prime-discrete` can be used to enforce the use of a discrete GPU using [PRIME](https://wiki.archlinux.org/title/PRIME). Depending on the details of your setup, you may need to change the contents of this file to "on" (to force discrete graphics) or "off" (to force integrated graphics).

On others, you may be able to the environment variable `DRI_PRIME=1` when running CodeOrbit to force the use of the discrete GPU.

If you're using an AMD GPU and CodeOrbit crashes when selecting long lines, try setting the `CODEORBIT_PATH_SAMPLE_COUNT=0` environment variable. (See [#26143](https://github.com/codeorbit-industries/CodeOrbit/issues/26143))

If you're using an AMD GPU, you might get a 'Broken Pipe' error. Try using the RADV or Mesa drivers. (See [#13880](https://github.com/codeorbit-industries/CodeOrbit/issues/13880))

If you are using `amdvlk` you may find that CodeOrbit only opens when run with `sudo $(which CodeOrbit)`. To fix this, remove the `amdvlk` and `lib32-amdvlk` packages and install mesa/vulkan instead. ([#14141](https://github.com/codeorbit-industries/CodeOrbit/issues/14141)).

For more information, the [Arch guide to Vulkan](https://wiki.archlinux.org/title/Vulkan) has some good steps that translate well to most distributions.

#### Forcing CodeOrbit to use a specific GPU

There are a few different ways to force CodeOrbit to use a specific GPU:

##### Option A

You can use the `CODEORBIT_DEVICE_ID={device_id}` environment variable to specify the device ID of the GPU you wish to have CodeOrbit use.

You can obtain the device ID of your GPU by running `lspci -nn | grep VGA` which will output each GPU on one line like:

```
08:00.0 VGA compatible controller [0300]: NVIDIA Corporation GA104 [GeForce RTX 3070] [10de:2484] (rev a1)
```

where the device ID here is `2484`. This value is in hexadecimal, so to force CodeOrbit to use this specific GPU you would set the environment variable like so:

```
CODEORBIT_DEVICE_ID=0x2484 CodeOrbit
```

Make sure to export the variable if you choose to define it globally in a `.bashrc` or similar.

##### Option B

If you are using Mesa, you can run `MESA_VK_DEVICE_SELECT=list CodeOrbit --foreground` to get a list of available GPUs and then export `MESA_VK_DEVICE_SELECT=xxxx:yyyy` to choose a specific device.

##### Option C

Using [vkdevicechooser](https://github.com/jiriks74/vkdevicechooser).

#### Reporting graphics issues

If Vulkan is configured correctly, and CodeOrbit is still not working for you, please [file an issue](https://github.com/codeorbit-industries/CodeOrbit) with as much information as possible.

When reporting issues where CodeOrbit fails to start due to graphics initialization errors on GitHub, it can be impossible to run the `CodeOrbit: copy system specs into clipboard` command like we instruct you to in our issue template. We provide an alternative way to collect the system specs specifically for this situation.

Passing the `--system-specs` flag to CodeOrbit like

```sh
CodeOrbit --system-specs
```

will print the system specs to the terminal like so. It is strongly recommended to copy the output verbatim into the issue on GitHub, as it uses markdown formatting to ensure the output is readable.

Additionally, it is extremely beneficial to provide the contents of your CodeOrbit log when reporting such issues. The log is usually located at `~/.local/share/CodeOrbit/logs/CodeOrbit.log`. The recommended process for producing a helpful log file is as follows:

```sh
truncate -s 0 ~/.local/share/CodeOrbit/logs/CodeOrbit.log # Clear the log file
CODEORBIT_LOG=blade_graphics=info CodeOrbit .
cat ~/.local/share/CodeOrbit/logs/CodeOrbit.log
# copy the output
```

Or, if you have the CodeOrbit cli setup, you can do

```sh
CODEORBIT_LOG=blade_graphics=info /path/to/CodeOrbit/cli --foreground .
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

### I can't open any files

### Clicking links isn't working

These features are provided by XDG desktop portals, specifically:

- `org.freedesktop.portal.FileChooser`
- `org.freedesktop.portal.OpenURI`

Some window managers, such as `Hyprland`, don't provide a file picker by default. See [this list](https://wiki.archlinux.org/title/XDG_Desktop_Portal#List_of_backends_and_interfaces) as a starting point for alternatives.

### CodeOrbit isn't remembering my API keys

### CodeOrbit isn't remembering my login

These feature also requires XDG desktop portals, specifically:

- `org.freedesktop.portal.Secret` or
- `org.freedesktop.Secrets`

CodeOrbit needs a place to securely store secrets such as your CodeOrbit login cookie or your OpenAI API Keys and we use a system provided keychain to do this. Examples of packages that provide this are `gnome-keyring`, `KWallet` and `keepassxc` among others.

### Could not start inotify

CodeOrbit relies on inotify to watch your filesystem for changes. If you cannot start inotify then CodeOrbit will not work reliably.

If you are seeing "too many open files" then first try `sysctl fs.inotify`.

- You should see that max_user_instances is 128 or higher (you can change the limit with `sudo sysctl fs.inotify.max_user_instances=1024`). CodeOrbit needs only 1 inotify instance.
- You should see that `max_user_watches` is 8000 or higher (you can change the limit with `sudo sysctl fs.inotify.max_user_watches=64000`). CodeOrbit needs one watch per directory in all your open projects + one per git repository + a handful more for settings, themes, keymaps, extensions.

It is also possible that you are running out of file descriptors. You can check the limits with `ulimit` and update them by editing `/etc/security/limits.conf`.

### No sound or wrong output device

If you're not hearing any sound in CodeOrbit or the audio is routed to the wrong device, it could be due to a mismatch between audio systems. CodeOrbit relies on ALSA, while your system may be using PipeWire or PulseAudio. To resolve this, you need to configure ALSA to route audio through PipeWire/PulseAudio.

If your system uses PipeWire:

1. **Install the PipeWire ALSA plugin**

   On Debian-based systems, run:

   ```bash
   sudo apt install pipewire-alsa
   ```

2. **Configure ALSA to use PipeWire**

   Add the following configuration to your ALSA settings file. You can use either `~/.asoundrc` (user-level) or `/etc/asound.conf` (system-wide):

   ```bash
   pcm.!default {
       type pipewire
   }

   ctl.!default {
       type pipewire
   }
   ```

3. **Restart your system**
