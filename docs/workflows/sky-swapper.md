# Skybox Mod implementation

This branch adds a live sky picker to ReRAC. It changes only the current level's sky;
the sky picker itself leaves lighting, fog, background clear colour, geometry, collision and
game progress with the host level. This branch also includes the independent
[lighting/fog editor](environment-mod.md). Original disc files are never overwritten. Settings are session-only and
reset to the original sky on a level change.

## Controls

Load a level, then press **F6** to open the panel. **Page Up / Page Down** cycle through
Original and the 19 planet sky sources. **F7** applies the selection; **F8** restores the
original sky and cancels a pending replacement. F7/F8 only act while the panel is open.
F6 closes the panel. Gameplay continues while the panel is open; these keys do not overlap
the existing movement/action controls. Controller and mouse UI are not implemented yet.

The old sky remains visible during loading. Errors appear in the panel and game log.
Title screens, movies, cutscenes and ship transitions defer swaps until normal level view.
Selecting the host planet is equivalent to restoring Original.

Optional environment settings:

| Variable | Meaning |
| --- | --- |
| `RC_SKY_SWAPPER=0` | Disable the panel and donor loading. |
| `RC_SKY_SOURCE=13` | Queue Gemlik Base's sky at startup (decimal 0..18, or `original`). |
| `RC_SKY_SWAPPER=1` | Explicitly enable even in deterministic capture runs; normally disabled there. |

The startup source is applied asynchronously. A screenshot taken at a fixed early frame
may precede its completion; wait for the `sky picker: applied ...` log before judging a
donor capture. An unmodified capture should leave the swapper disabled.

PowerShell example from this checkout:

```powershell
$env:RC_DATA_DIR = 'C:\path\to\rerac\games\rac1\data'
$env:RC_LEVEL = '1'
cargo dev
```

Use data extracted from the disc supported by ReRAC (NTSC-U SCUS-97199 v1.00).
OpenRAC's PAL data is not a replacement for that extraction.

## Packaging and building in GitHub

`tools/package/skybox-mod.sh` builds the runtime and extractor, preserves the launcher
manifest contract, and wraps them with the player guide and ISC license. It includes no
game data. This is a compiled mod edition: the planned dynamic plugin loader in upstream
`docs/plan/mods.md` is not implemented. Players need no compiler or source checkout.

Push `work/environment-editor` to the fork to trigger **Skybox and Lighting Mod preview**.
The workflow type-checks, lints, and runs the unit and shader tests on Linux, then builds a Windows
ZIP in a separate job. The workspace trace tools use Unix sockets and cannot be included
in the Windows workspace checks; the Windows release build checks the actual shipped
runtime and extractor. Successful jobs prepare a **draft prerelease** on
`crystalfissure/rerac`, with the ZIP and its SHA-256 checksum. Gameplay testing is required
before publishing that draft. It is not an automatic public release.

Manual reruns are also supported under
**Actions → Skybox and Lighting Mod preview → Run workflow**, selecting this branch.
If GitHub requires the workflow to exist on the default branch before showing its manual
button, merge the workflow file into the fork's default branch first.

Unzip the downloaded workflow artifact to obtain its inner `rerac-skybox-mod-*.zip`. In ReRAC
Launcher, use Settings → Version Management → Development → Install from zip and set it
active. This branch uses version `0.3.3-sky.3` to distinguish it from the upstream build.
Alternatively use Add build folder with an unpacked package in its own folder.

The fork must have GitHub Actions enabled; GitHub may require the owner to enable Actions
first. Mod tags start with `skybox-v` so they do not trigger upstream's `v*` release workflow.

## Implementation

- `sky_swap.rs`: single background loader, source validation, generation-bound requests,
  picker, sky-only cleanup, restoration, and transient donor memory.
- `sky_render.rs`: reusable sky startup schedule; reuse the existing sky camera and reset
  the first-frame clear gate. Only the ordinary level's shells receive `SkyOwned`.
- `sky_stars.rs`: donor overlay tables are prepared on the worker; the host's original
  star simulation continues invisibly on the shared gameplay RNG. Donor stars use a
  private RNG. Restore resumes the host simulation rather than regenerating its stars.
- No donor-level actors, collision, audio or gameplay are loaded. A temporary in-memory
  lump store is dropped after parsing, avoiding an ever-growing global donor cache.
- GPU objects are owned by Bevy handles; removing donor entities and simulation handles
  lets Bevy reclaim their assets. There is one pending worker and no all-planets GPU cache.
- Requests are discarded after a level generation change or Restore. A new Apply queues
  the newest choice behind the single in-flight worker; it does not create more threads.

## Validation before calling this playable

Run `cargo check-all`, `cargo clippy-all`, and `cargo xtask test-quick` using the upstream
test workflow. The sky-specific subset is `cargo xtask test-quick rc-engine --filter sky_`.

With real game data and a GPU, check:

1. Novalis → Gemlik Base, then Restore; terrain, HUD and position stay unchanged.
2. Original star host → starless donor and back; stars return and animate normally.
3. Blarg Station (06) and Drek's Fleet (17) as donors (overlay-dependent stars).
4. Orxon and Rilgar as donors (clearing/transparent sky differences).
5. Repeated donor changes, including several before loading finishes; no extra cameras,
   orphaned draws, unbounded memory growth or old-sky trails.
6. Restore while loading, invalid/missing donor data, and travel during loading.
7. Movies, cutscenes, title return and planet travel; no swap applied to those special skies.

These runtime checks require the user's extracted data; unit tests cannot certify the
visual result. No game data belongs in this repository or the workflow artifact.

## Reference reviewed

Base: `re-rac/rerac` commit `a9f976c02d92ef28b89648e0437db64619c64779`.
OpenRAC reference: `OpenRAC/rac1-decomp` commit
`9a8143337c49794a9b6588d253f79683ad1484ce`, especially `src/game/loaders.c`,
`src/game/skyfunc.c`, the shared sky overlay code and `tools/extract/sky.py`.
It corroborates the shell/texture/relocation model, but targets PAL v2.00. No OpenRAC
implementation was copied into this Rust change; existing ReRAC parsers/renderers are reused.

Lighting/fog controls are now implemented in the separate F5 panel. Future work includes
saved planet presets and a richer sky selector, after the previews have been tested in game.
