# Sky swapper validation — 15 October 2026

Prototype version: **0.3.3-sky.1**, based on upstream
`a9f976c02d92ef28b89648e0437db64619c64779`.

## Checks performed

| Check | Result |
| --- | --- |
| `cargo check-all` | Passed: workspace and all targets, including tests. |
| `cargo clippy-all` | Passed; one pre-existing `needless_borrows_for_generic_args` warning at `tools/trace/src/class_census.rs:843`, also reported for that crate's test target. |
| `cargo xtask test-quick` | Engine passed (76 passed, 1 ignored); wider run stopped at two existing PINE Unix-socket tests with `PermissionDenied` from `UnixListener::bind`. |
| Same unit suite, explicitly excluding those two socket tests | **1,028 passed, 0 failed, 3 ignored, 2 filtered out** across the workspace. |
| Seven newly added regression tests | All passed. |
| Mod packaging with synthetic executables on Linux | Passed: common ZIP root, launcher manifest preserved, player guide substitutions, ISC license, release notes and SHA-256 sidecar. An unexpected game-data file was correctly rejected. |

The second test invocation was:

```sh
cargo xtask test-quick -- --skip pine::tests::read_many_framing --skip pine::tests::batched_read_framing
```

The excluded tests exercise the PCSX2 PINE trace client's socket framing, not sky swapping.
The environment forbids their Unix-socket operation; no socket restrictions were bypassed
and no upstream tests were edited or disabled in source. CI runs the sky filter through
the upstream xtask runner on Linux without these local exclusions; Windows builds the
shipped runtime and extractor, excluding the Unix-only developer tracing tools.

## Regression coverage added

- Invalid source IDs and selection wraparound.
- Sky-owned entities are removed while unrelated entities and the sky camera survive.
- A completed request from an earlier level generation is discarded.
- A failed donor load retains the active selection; Restore cancels a finished request.
- Live replacement resets the first-frame clear gate without a level-generation change.
- Original star simulation progress is retained when restored.
- Over 360 ticks for each of five host sky types, changing donors (including a starless
  donor) leaves the shared gameplay RNG and original star records identical to a baseline.

## Build environment

Linux x86_64, Rust 1.99.0. Rust and native build dependencies were provisioned in the
temporary environment. Native libraries were unpacked into a temporary sysroot; missing
system packages were not represented by fake pkg-config files or mocked libraries.

Checks/lints used the repository's optimisation settings, with debug information and
incremental compilation disabled. Test compilation temporarily used opt-level 0 for
the workspace and dependencies to reduce the cost of the cold Bevy build. The temporary
Cargo configuration was restored afterward; it is not part of the patch. The test runner
still used the upstream workspace and `rc-engine/dev` feature selection.

## Follow-up Windows build

The first preview subsequently passed cloud checks and built real Windows executables in
[Actions run 38686301734](https://github.com/crystalfissure/rerac/actions/runs/38686301734).
Its ZIP payload, manifest, checksum and PE binaries were inspected. This does not establish
visual/gameplay correctness. The lighting/fog follow-up is described in `environment-mod.md`.

## Still unverified

- Actual rendering with the user's supported extracted game data.
- Repeated-swap GPU memory behaviour, transparency/clear artifacts, and loading pauses.
- Planet travel, cutscenes, title-world return and donor overlay effects in a running game.

This is a compile-checked, unit-tested **binary preview**, not a visually verified
playable release. See `sky-swapper.md` for controls, build instructions and runtime checks.

The mod packaging fixture uses fake executables only to exercise archive assembly and
manifest checks; it is not a playable build and is never uploaded. CI builds the actual
Windows executables and parses the standalone PowerShell starter with Windows PowerShell.
