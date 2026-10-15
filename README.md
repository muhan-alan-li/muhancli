# muhan — portfolio in the terminal

```sh
muhan about
muhan experience
muhan experience apera-ai
muhan projects --tag react
muhan projects dune-imperium
muhan skills --group Languages
muhan contact
muhan --json experience
muhan --plain projects
muhan version
```

Install (prebuilt binary, when releases exist):

```sh
curl -fsSL https://raw.githubusercontent.com/muhan-alan-li/muhancli/main/install.sh | sh
```

From source:

```sh
cargo install --git https://github.com/muhan-alan-li/muhancli
```

Flags: `--json` (stable machine output), `--plain` (no color), `NO_COLOR` respected.
