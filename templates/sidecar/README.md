# __PKG__

__DESC__

A sidecar plugin for [gray](https://github.com/vstaln/gray), made with
[gray-maker](https://github.com/vstaln/gray-maker).

## Install

```sh
gray plugin install __KEY__
```

## Develop

```sh
cargo test
gray-maker check        # entry point + manifest handshake
gray-maker ship         # build → release → publish to the gray registry
```

Bump `version` in `Cargo.toml` before each `ship`; the registry refuses to
republish a version.
