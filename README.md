# muhan — portfolio in the terminal

Explore Muhan Li's experience, projects, and skills from your terminal. Installs a
single executable with the portfolio content bundled inside. No Rust installation,
account, or server connection is needed to use a prebuilt release.

## Linux and macOS

Run in Bash or your usual terminal:

```sh
curl -fsSL https://raw.githubusercontent.com/muhan-alan-li/muhancli/main/install.sh | bash
```

Then explore:

```sh
export PATH="$HOME/.local/bin:$PATH"
muhan
muhan about
muhan --help
```

The installer detects Intel/AMD x64 or ARM64 (including Apple Silicon), verifies
the release's SHA-256 checksum, and installs to `~/.local/bin` without `sudo`.
Linux releases use musl so they also work on distributions without glibc, such as
Alpine. macOS releases target macOS 11 or newer.

If needed, it adds a PATH entry to your Bash or Zsh startup files for future
terminals. The `export` above makes it available immediately in your current
terminal. Fish users can run `fish_add_path ~/.local/bin` instead.

If curl is unavailable, download and run with wget:

```sh
wget -qO install-muhan.sh https://raw.githubusercontent.com/muhan-alan-li/muhancli/main/install.sh
sh install-muhan.sh
```

To choose a version or installation directory, download the script first:

```sh
curl -fsSLo install-muhan.sh https://raw.githubusercontent.com/muhan-alan-li/muhancli/main/install.sh
VERSION=v0.1.0 DEST="$HOME/.local/bin" sh install-muhan.sh
```

Set `NO_MODIFY_PATH=1` to leave shell startup files untouched. `REPO` can override
the GitHub repository when installing from a fork. A SHA-256 tool (`sha256sum` or
`shasum`) and `tar` are required; a missing checksum or a mismatch stops the install.

## Windows PowerShell

Run in Windows PowerShell 5.1 or PowerShell 7 on 64-bit Windows 10/11:

```powershell
Invoke-RestMethod https://raw.githubusercontent.com/muhan-alan-li/muhancli/main/install.ps1 | Invoke-Expression
muhan
muhan about
muhan --help
```

The installer detects x64 or ARM64, verifies the SHA-256 checksum, and installs
`muhan.exe` to `%LOCALAPPDATA%\muhan\bin`. It updates PATH for your current
PowerShell session and future terminals. No administrator access, Rust toolchain,
or separate Visual C++ runtime installation is required.

To review the script or pass options, download it first and invoke its contents:

```powershell
Invoke-WebRequest -UseBasicParsing https://raw.githubusercontent.com/muhan-alan-li/muhancli/main/install.ps1 -OutFile install-muhan.ps1
Get-Content .\install-muhan.ps1
& ([scriptblock]::Create((Get-Content .\install-muhan.ps1 -Raw))) -Version v0.1.0 -Destination "$env:LOCALAPPDATA\muhan\bin"
```

`-NoModifyPath` skips PATH changes; invoke the executable by its full path in that
case. `-Repository` selects a fork. The installer also accepts `VERSION`, `DEST`,
`REPO`, and `NO_MODIFY_PATH=1` environment variables.

## Explore

Run `muhan` alone for an introduction and suggested commands.

```sh
muhan about
muhan experience
muhan experience apera-ai
muhan projects
muhan projects --tag react
muhan projects dune-imperium
muhan skills --group Languages
muhan contact
muhan version
```

Use `--json` for machine-readable output or `--plain` to disable colors.
`NO_COLOR` is respected, and piped output has no color codes.

```sh
muhan --json experience
muhan --plain projects
```

## Tab completion

Completion is available for Bash, Zsh, Fish, PowerShell, and Elvish. It suggests
commands, flags, experience IDs, project slugs, project stack tags, and skill
groups. For example, type `muhan experience ap` then press Tab to get `apera-ai`.
Suggestions come from the installed binary and work offline.

Enable it in your current shell after putting `muhan` on PATH:

```bash
# Bash (including macOS's bundled Bash 3.2)
eval "$(muhan completions bash)"
```

```zsh
# Zsh: initialize completion first if your shell setup hasn't already done so.
autoload -Uz compinit
compinit
eval "$(muhan completions zsh)"
```

```fish
# Fish
muhan completions fish | source
```

```powershell
# Windows PowerShell 5.1 or PowerShell 7
muhan completions powershell | Out-String | Invoke-Expression
```

For future sessions, add the corresponding activation line after your PATH setup
in `~/.bashrc`, `${ZDOTDIR:-$HOME}/.zshrc`, `~/.config/fish/config.fish` (or your
Fish config under `XDG_CONFIG_HOME`), or the PowerShell `$PROFILE` file. In Zsh,
place it after `compinit`. Bash login shells should load `~/.bashrc` from their
login profile. PowerShell profiles are specific to each host, so configure both
Windows PowerShell and PowerShell 7 if you use both. The installers configure
PATH; completion activation is a separate, one-time setup.

Generate the adapter on each shell startup as shown above, so updates refresh
both the adapter and the suggestions. To remove completion, delete its activation
line and start a new shell. Elvish users can add
`eval (muhan completions elvish | slurp)` to their shell config.

## Update or uninstall

Run the same installer again to update to the latest release. Installed versions
work offline; updating also refreshes the bundled portfolio content.

To uninstall on Linux/macOS:

```sh
rm "$HOME/.local/bin/muhan"
```

You can also remove the `# muhan portfolio CLI` entry and its following PATH line
from the shell startup files reported by the installer.

To uninstall on Windows:

```powershell
Remove-Item "$env:LOCALAPPDATA\muhan\bin\muhan.exe"
```

Remove `%LOCALAPPDATA%\muhan\bin` from your user PATH using Windows' Environment
Variables settings. Adjust these paths if you selected a custom destination.

## Build from source

With a Rust toolchain installed:

```sh
cargo install --locked --git https://github.com/muhan-alan-li/muhancli
```

## Releases and development

**The binary installers require a published GitHub release.** Until the first
release is published, use the source installation above. The release workflow
builds and tests native x64 and ARM64 binaries for Linux, macOS, and Windows,
packages them as `.tar.gz` or `.zip`, and attaches a `.sha256` file to each archive.

For a release, set the package version in `Cargo.toml`, refresh `Cargo.lock`, and
push a matching version tag such as `v0.1.0`. The workflow verifies the tag matches
the package version and publishes the GitHub release only after all six builds
and installer checks pass. Branch pushes and pull requests build and test without
publishing. `workflow_dispatch` can build downloadable CI artifacts without a tag.

Local checks on Linux/macOS:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked --release
python3 tests/install_unix.py
```

Installer tests simulate release downloads locally and execute the installed
binary. Windows CI runs `tests/install_windows.ps1` under both Windows PowerShell
5.1 and PowerShell 7 against the packaged Windows build.
