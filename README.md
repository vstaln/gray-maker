# gray-maker

Make [gray](https://github.com/vstaln/gray) plugins: scaffold one, prove gray
can start it, build a static binary, release it on GitHub, and publish it to
the gray registry — so `gray plugin install <name>` just works.

```sh
gray plugin install maker
gray-maker new weather --description "Weather lookups for gray agents"
cd ~/grayplugins/gray-weather
# …write the plugin in src/main.rs…
cargo test
gray-maker ship            # check → build → release → publish
```

In the REPL, `/maker new <name>` scaffolds right away; `/maker ship` (and the
other long steps) hand the agent a prompt to run them in bash, because gray
gives slash commands 30 seconds.

## Commands

| command | does |
|---|---|
| `new <name> [--dir D] [--description T] [--no-repo]` | Scaffold `~/grayplugins/gray-<name>` from the sidecar template (wire handshake, one example tool, one slash command, tests, MIT), commit it with its `Cargo.lock`, and create + push `vstaln/gray-<name>`. |
| `check` | Debug-build, stage the binary the way the tarball will, start it with gray's rule (`plugin.sh`, else the single executable) and complete the `plugin/manifest` handshake. Fails if the manifest name ≠ registry key or the version ≠ `Cargo.toml`. |
| `build [--remote HOST]` | Static `x86_64-unknown-linux-musl` release build — locally (`cargo zigbuild` when present) or on `HOST` via `git archive HEAD \| ssh`. Refuses a dirty tree, refuses a dynamically linked binary, and writes a byte-reproducible `dist/gray-<name>-<version>.tar.gz`. |
| `release` | Requires HEAD pushed. Creates release `v<version>` (or reuses it), uploads the tarball, then re-downloads the public URL until it serves the same sha256. A different tarball for an existing version is refused — bump the version. |
| `publish` | `POST /api/plugins/submit` with the token from `gray account login`. The registry hashes the tarball itself and rejects a mismatch with ours. |
| `ship [--remote HOST]` | All of the above, stopping at the first failure. |

### Plugins that need flags

`gray` starts a plugin with no arguments. If yours needs some (say
`my-plugin sidecar`), commit a `plugin.sh` next to `Cargo.toml`:

```sh
#!/bin/sh
exec "$(dirname "$0")/my-plugin" sidecar "$@"
```

`build` ships it in the tarball and `check` starts the plugin through it.

### Remote builds

`--remote HOST` needs `~/.cargo/bin/cargo`, `cargo-zigbuild`, and zig (on
`PATH` or at `~/build/zig/zig`) on the build box. Sources go to
`~/build/gray-maker/<package>`, artifacts to `~/build/gray-maker/target`.

## Environment

- `GRAY_REGISTRY_URL` — registry base (default `https://gray.alignment.id/api`;
  `http://` only on loopback).
- `GRAY_HOME` — where `registry-token.json` lives (default `~/.gray`).

## Limits

Linux x86_64 only for now: the plugin index holds one tarball per plugin.
