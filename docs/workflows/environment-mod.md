# Lighting and fog mod preview

Mod 0.3.0-preview.1 / runtime 0.3.3-sky.3 adds live appearance controls alongside the
sky picker. This is a separate preview branch (`work/environment-editor`) based on the
skybox preview; it does not replace the first skybox-only build.

## Player controls

Load a level and press F5. Click the panel's buttons, or use Home/End to select a row
and [ / ] to decrease/increase it. Delete resets the selected row (all three channels for fog colour). F4 compares the
original lighting/fog with the current edits; F3 resets all lighting/fog controls.
These keys act while the panel is open. F5 closes it; F6 switches to the sky picker.
Gameplay continues while editing. Escape releases a captured mouse cursor.

| Control | Range | Original |
| --- | --- | --- |
| Brightness | 0–200% | 100% |
| Vertex red, green, blue | 0–200% each | 100% each |
| Shadow lift | 0–100% additive vertex colour | 0% |
| Fog strength | 0–200% opacity multiplier | 100% |
| Fog distance | 0.25–4 times current level/zone distances | 1 |
| Fog red, green, blue | 0–255 each | Current game fog colour |

The first fog colour edit starts from the current native colour. Resetting any fog
colour row clears the custom RGB triple, restoring all three live game channels.
Comparison affects lighting and fog only; F8 in the sky picker restores the sky.
Settings are session-only and reset on level change. Movies, cutscenes, title views
and ship transitions temporarily bypass these overrides.

## Rendering boundary

Vertex RGB is adjusted after native lighting and LOD colour blending, before texture
modulation and fog. The same transform covers terrain, static objects, shrubs and
billboards, world actors and their metal pass. It preserves alpha. Default or inactive
settings explicitly bypass the transform rather than relying on neutral floating-point
math. The original asset bytes remain untouched.

The initial scope excludes sky colours, HUD and menu model lighting, and the separate
water/particle/effect vertex-colour pipelines. World actor effects drawn through the
ordinary actor material do receive the adjustment. This is a whole-level rendering
override, not a permanent bake or a per-vertex painting tool.

Fog edits operate on a display copy of the current game fog. Native zone and underwater
updates continue. Terrain LOD, particle clipping and simulation keep their original
inputs, including when visual fog strength is zero. Ordinary water/effects that consume
the shared displayed fog also see it; specialized sea passes with local fog stay native.
Removing fog does not reveal geometry that the engine has already culled.

Menu/vendor preview models carry an opt-out bit in the existing instance record. Their
lighting and fog stay native while the world behind them can retain its edits.

## Integration

- `mod_ui.rs`: binds both mod panels to the window presentation camera. Bevy UI mouse
  focus requires a window target; the game camera renders into an image. Fits panels to
  smaller windows without changing the game camera, HUD, render resolution or draw distances.
- `environment_mod.rs`: session settings, live panel, comparison, bounded inputs and
  level-unload reset; `RC_ENVIRONMENT_MOD=0` disables it. Deterministic runs disable it
  unless explicitly enabled with `RC_ENVIRONMENT_MOD=1`.
- `fog_state.rs`: derives displayed fog without mutating native game globals or culling.
- `game_camera.rs`: uploads one shared 112-byte storage buffer after fog preparation.
  Vectors are display colour (0), display parameters (16), graphics options (32),
  light multiplier/enable (48), light lift (64), native colour (80), native parameters (96).
- Water, sea and cached effect materials refresh fog even while their animation counters
  are paused; local sea fog remains separate. Draw preparation follows the fog update.
- Six world material shaders and `world_lights.wgsl`: vertex transform and menu opt-out.
  Existing bindings and draw ordering remain intact.
- `environment_shader_tests.rs`: compiles 27 alpha/blend combinations of the actual WGSL,
  with small typed substitutes for Bevy view imports; no GPU or game assets are required.

## Initial lighting/fog validation — 15 October 2026

- Workspace/all-target type check passed.
- Workspace/all-target Clippy passed with the existing unrelated trace-tool
  `needless_borrows_for_generic_args` warning in `class_census.rs`.
- Unit suite: **1,045 passed, 0 failed, 3 ignored, 2 filtered out**. This includes
  **17 new environment regressions** and 27 compiled shader variants within the shader test.
- The two filtered PINE Unix-socket tests cannot bind sockets in this local environment;
  the cloud workflow runs the complete unit suite without these exclusions. No test source
  or upstream guards were disabled. Existing unrelated personal-path guard failures are
  documented in the sky preview review, outside the quick unit tier.
- Linux Rust 1.99.0, real native dependencies in a temporary sysroot. Test-only dev
  optimisation overrides were restored after running; the shipped build uses release settings.

## Preview 0.3 review fixes — 15 October 2026

- Mod panels target the window presentation camera. Bevy 0.19's `ui_focus_system`
  ignores image targets, so 0.2's keyboard editor worked but its mouse buttons did not.
- Panel transforms fit smaller windows while retaining the top-left anchor and native
  pointer coordinates, independently of the game render resolution.
- Row reset exits original comparison, matching ordinary adjustment behavior.
- Selecting the host planet normalizes to Restore before waiting for a donor worker.
- Local quick suite: **1,049 passed, 0 failed, 3 ignored, 2 filtered** (the same locally
  unavailable PINE socket tests). Four new regressions include the real Bevy focus
  system: reproduce the ignored image-target click, retarget, and assert a button press.
  The overlay layout regression checks Bevy's affine transform, bounds and anchor.
- Workspace/all-target type check and Clippy passed (only the existing trace-tool
  needless-borrow warning). The cloud workflow runs the complete unit
  suite and builds a separate 0.3 draft package before PC visual testing.

## Validation and PC test pass

Use `cargo check-all --locked`, `cargo clippy-all --locked`, and `cargo xtask test-quick`.
The targeted subset is `cargo xtask test-quick rc-engine --filter environment_`.
The cloud workflow builds the Windows runtime/extractor and attaches a separate draft
preview ZIP only after the Linux checks and Windows package job succeed.

Before calling the preview visually verified, test with supported extracted game data:

1. Start in Novalis at defaults. F4 comparison should make no visible difference.
2. Lower brightness, increase shadow lift, and change individual RGB channels. Check
   terrain, props, plants, Ratchet and shiny actors; HUD and sky should retain their colours.
3. Set fog strength to zero, then increase it, move its distance and change RGB. Watch
   terrain seams/LOD and transparent objects; there should be no shader errors or flicker.
4. Open pause and vendor menus with strong RGB/fog edits. Model previews should retain
   their native appearance; close menus and confirm the world edits persist.
5. Move through a fog zone and enter/leave water. Compare and reset: native fog should
   resume its current state, without restoring a stale colour from before the transition.
6. Combine these controls with a donor sky, then compare lighting/fog and restore sky
   independently. Travel to another level: both editors should return to defaults.
7. Test a movie/cutscene, return to the title, and repeatedly open/close both panels.

Actual gameplay and GPU rendering require the user's PC and game data. Shader and unit
checks cannot establish the final visual result or replace this pass.
