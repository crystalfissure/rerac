# Skybox Mod 0.2.0-preview.1 — lighting and fog

An unofficial ReRAC mod that lets you choose another planet's sky and adjust
level brightness, tint and fog while playing.
This download includes the modded runtime: no source code or compiler is needed.

Download the **`rerac-skybox-mod-0.2.0-preview.1-windows-x86_64.zip`** asset for
Windows. Extract the complete ZIP, then double-click **Play-Skybox-Mod.cmd**.
Choose your existing extracted data folder, or let the starter extract your ISO
into a new empty folder and start the game.

Windows also needs the **Microsoft Visual C++ Redistributable (x64)**. Install it
from [Microsoft](https://aka.ms/vc14/vc_redist.x64.exe) if it is missing; this
resolves a missing `VCRUNTIME140.dll` error.

If you already use ReRAC Launcher, you can instead open **Settings → Version
Management → Development → Install from zip…**, import the ZIP, then **Set active**
on **ReRAC 0.3.3-sky.2**.
Your existing compatible ReRAC game data can be reused. New installations need
your own supported NTSC-U SCUS_971.99 v1.00 disc image.

Load a level and press **F6**. Choose a sky with **Page Up / Page Down**, apply it
with **F7**, or restore the original with **F8**. Keep the panel open for F7/F8.
Sky changes reset when changing levels.

**New: press F5 for lighting and fog.** Use **Home / End** to select a row,
**[ / ]** to adjust it, and **Delete** to reset it. **F4** compares original
lighting/fog while keeping your edits; **F3** resets all lighting/fog settings.
Keep the panel open for editing keys, or use its clickable buttons. Gameplay
continues while the editor is open; **Esc** releases the mouse cursor.

- Adjust vertex brightness, separate RGB tint and shadow lift on terrain,
  props, shrubs and actors, including metallic actors. Sky, HUD, menu previews,
  water and effect vertex colours are excluded.
- Adjust fog strength, distance and RGB colour. Fog edits affect rendering;
  object draw distance, detail levels, clipping and physics retain their normal
  behaviour. Special sea effects retain their own fog, and menu previews keep
  their normal lighting and fog.
- Edits are session-only and reset on level changes. They temporarily pause
  during title screens, movies, cutscenes and ship travel. Opening another
  editor closes the previous panel without discarding its settings.

Resetting a fog-colour row restores all three channels to the level colour.
F3/F4 affect lighting and fog; sky restoration remains **F8** in the sky picker.

This is an early preview; visual gameplay validation with real game data is
still required. It bundles its own runtime, so it cannot automatically combine
with other mods that replace the runtime. No original game assets are included.

Please share problems in the current preview's
[pull request](https://github.com/crystalfissure/rerac/pulls) with the host
planet, chosen sky, lighting/fog values, steps to reproduce and
**skybox-mod-last-run.log** from the mod folder (or the game log from the
launcher's **Help → Logs folder**).
The ZIP includes the full player guide and ISC license.
