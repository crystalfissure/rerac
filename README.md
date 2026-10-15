<p align="center">
  <img src="docs/assets/rerac-logo.png" alt="ReRAC" width="480">
</p>

<h1 align="center">ReRAC Skybox Mod</h1>

<p align="center">CrystalFissure's skybox, lighting and fog mod for ReRAC.</p>

<p align="center">
  <a href="https://discord.gg/v2Ek44kdyu">Discord</a> ·
  <a href="https://re-rac.github.io">Website</a> ·
  <a href="https://github.com/re-rac/rerac-launcher">Launcher</a>
</p>

## Please read first

This fork adds an in-game picker for skies from the 19 planets. Load a level, press **F6**,
choose a source with **Page Up / Page Down**, and press **F7** to apply or **F8** to restore.
Press **F5** for live brightness, RGB tint, shadow lift and fog controls. Use
**Home / End** to select a row, **[ / ]** to adjust, **Delete** to reset that control,
**F4** to compare original lighting/fog, and **F3** to reset all lighting/fog edits.
Keep the relevant panel open for editing keys. The lighting/fog editor also has
clickable controls; **Esc** releases the cursor. Gameplay continues during editing.
Opening one panel closes the other without discarding its settings. All edits reset
when you change planets; saved presets are not yet supported.

Vertex-colour controls affect terrain, props, shrubs and actors, including metallic
actors. They exclude sky, HUD, menu previews, and water/effect vertex colours.
Fog strength, distance and RGB edits change rendering while preserving object draw
distance, detail levels, clipping and physics. Special sea effects keep their own
fog; menu previews keep their normal lighting and fog. Lighting/fog edits temporarily
pause in title screens, movies, cutscenes and ship travel.

The mod ships as a **prebuilt Windows package**; players do not need Rust or source code.
Extract the ZIP and double-click **Play-Skybox-Mod.cmd**. Its dialogs let you select existing
ReRAC data or extract your supported disc ISO. Windows also needs the
[Microsoft Visual C++ Redistributable (x64)](https://aka.ms/vc14/vc_redist.x64.exe)
if it is not already installed; this resolves a missing `VCRUNTIME140.dll` error.
The launcher is optional.
ReRAC does not yet implement its planned plugin loader, so the package includes a runtime
with the mod built in. It installs as its own version (`0.3.3-sky.3`) in the launcher's
Development section. It cannot currently be stacked with other mods that replace the runtime.

Preview 0.3 fixes mouse input for the lighting/fog buttons, fits the mod panels to smaller
windows, makes row resets exit comparison, and restores the host sky immediately while
another sky is loading.

**Status: 0.3.0-preview.1, awaiting real-game visual testing.** The
[Skybox and Lighting Mod preview workflow](https://github.com/crystalfissure/rerac/actions/workflows/sky-swapper.yml)
checks the code and builds the Windows ZIP. Successful builds prepare a draft prerelease for
the owner to test before sharing. This README does not mean a build has passed or a release
has been published; consult the workflow and [releases](https://github.com/crystalfissure/rerac/releases).

See the [player guide](docs/mods/skybox/README.txt),
[sky implementation notes](docs/workflows/sky-swapper.md),
[lighting/fog notes](docs/workflows/environment-mod.md), and
[validation record](docs/workflows/sky-swapper-validation.md).

> [!WARNING]
> You need your own copy of the game: no game assets are included here. ReRAC currently supports one disc, listed on
> the [supported versions](https://re-rac.github.io/docs/supported-versions/) page. ReRAC is an unofficial fan project
> and is not affiliated with or endorsed by Sony Interactive Entertainment or Insomniac Games.

## About ReRAC

ReRAC rebuilds Ratchet & Clank (2002, PlayStation 2) as a native PC game, written in Rust on the Bevy engine. The
game's systems are ported and run natively, with no emulator in between. The goal is a port
that looks, sounds and plays like the original.

## Status

ReRAC is in early, active development and is not yet playable from start to finish. All levels load and render, and
more of the game comes to life with each update. See the [status page](https://re-rac.github.io/status/) for what
works today.

## Developing the mod

Players use the packaged download. The following instructions are for contributors.

You need Rust, installed through [rustup](https://rustup.rs), and an image (`.iso`) of your own disc. macOS on Apple
Silicon is the only platform tested so far. Run every command from the repository root.

First, extract the game data from your disc into `extracted/`. This builds `rerac-extract`, checks that your disc is
supported, and verifies every extracted file:

```sh
cargo xtask regen-data --iso "<path to your disc image>.iso"
```

Then build and run the game:

```sh
cargo dev
```

The first build takes a few minutes; later rebuilds take seconds. Tests run through `cargo xtask`:

```sh
cargo xtask test-quick      # fast unit tests
cargo xtask test-job hero   # unit tests plus one area's integration tests
cargo xtask test-full       # the full suite
```

More detail is in [`docs/workflows/`](docs/workflows/), for example
[`game-data.md`](docs/workflows/game-data.md) for the extractor and
[`dev-switches.md`](docs/workflows/dev-switches.md) for debugging options.

## Repository layout

| Folder | Contents |
|---|---|
| `crates/` | The game itself: file formats, game logic, the engine (`rerac`) and the extractor (`rerac-extract`) |
| `tools/` | Development tools that never ship, including `cargo xtask` |
| `docs/` | [Developer documentation](docs/): format specs, plans and workflows |

## Related

- [ReRAC Launcher](https://github.com/re-rac/rerac-launcher), which installs and runs the game.
- [re-rac.github.io](https://re-rac.github.io), the project website.

## License

ReRAC is released under the ISC license. See [LICENSE](LICENSE).
