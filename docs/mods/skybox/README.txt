ReRAC Skybox Mod @MOD_VERSION@
Unofficial preview by CrystalFissure, built on ReRAC @RUNTIME_VERSION@.

Choose another planet's sky, then adjust level brightness, tint and fog while
playing Ratchet & Clank.
This download includes the modded game runtime. No source code, compiler, or
developer tools are needed to play. The Windows package includes a starter.
If you already use ReRAC Launcher, you can also install it as a separate game
version; the launcher calls this section "Development".

REQUIREMENTS

- Your own supported Ratchet & Clank disc image: NTSC-U SCUS_971.99 v1.00.
  An existing compatible ReRAC data installation can be reused.
- Download the package for your operating system and processor.
- Windows: Microsoft Visual C++ Redistributable (x64). If it is not already
  installed, get Microsoft's installer here:
  https://aka.ms/vc14/vc_redist.x64.exe
  This also resolves a missing VCRUNTIME140.dll error when starting the game.

WINDOWS: EXTRACT AND PLAY

1. Download rerac-skybox-mod-@MOD_VERSION@-windows-x86_64.zip.
   Use the mod ZIP, not GitHub's "Source code" downloads.
2. Extract the complete ZIP into a folder. Keep all the files together.
3. Double-click Play-Skybox-Mod.cmd in that folder.
4. Choose Yes if you already have extracted ReRAC data, then select the folder
   containing toc.bin and extract-info.json. Choose No to select your supported
   ISO and extract it into a new empty folder, then play.

The starter opens file/folder dialogs; it does not need administrator access.
Keep the extracted data for future sessions and choose Yes next time. Game
output is saved as skybox-mod-last-run.log beside the starter when writable.

OPTIONAL: INSTALL IN AN EXISTING RERAC LAUNCHER

Launcher project: https://github.com/re-rac/rerac-launcher

1. Download rerac-skybox-mod-@MOD_VERSION@-<os>-<arch>.zip.
   Use the mod ZIP, not GitHub's "Source code" downloads.
2. Open ReRAC Launcher. Go to Settings > Version Management > Development.
3. Choose "Install from zip..." and select the downloaded mod ZIP.
4. Find "ReRAC @RUNTIME_VERSION@" in the list and choose "Set active".
5. Select Ratchet & Clank in the sidebar. If your game data is already
   installed, choose Play. Otherwise choose "Install via ISO" and select
   your supported disc image once, then Play.

If you downloaded an Actions artifact instead of a release download, unzip
the artifact first. Import the inner rerac-skybox-mod-*.zip into the launcher.

SKYBOX CONTROLS

Load a level, then use the keyboard:

  F6                   Open or close the sky picker
  Page Up / Page Down  Choose Original or one of 19 planet sky sources
  F7                   Apply the selected sky
  F8                   Restore the current level's original sky

Keep the picker open to use F7 and F8. On some keyboards, hold Fn to use the
function keys. Play continues while the picker is open.

The current sky stays visible while its replacement loads. The panel shows
loading status or an error. F8 cancels a pending replacement as well as
restoring the original sky. Selecting the current planet restores Original.
The sky resets when you change levels. Swaps wait during movies, cutscenes
and ship travel until ordinary level view resumes.

LIGHTING AND FOG CONTROLS

Load a level and press F5 to open the lighting and fog editor. Changes are live:

  F5                   Open or close the lighting and fog editor
  Home / End           Select the previous / next control row
  [ / ]                Decrease / increase the selected value
  Delete               Reset the selected control
  F4                   Compare original lighting/fog; press again for your edits
  F3                   Reset all lighting and fog controls

Keep the editor open to use these editing keys. You can also click a row's
minus, plus, Reset or Level RGB button, or the Compare, Reset all and Close buttons.
Esc releases the mouse cursor. Gameplay continues while you make changes.

Brightness and the separate red, green and blue controls run from 0% to 200%.
Shadow lift runs from 0% to 100% and adds light to dark vertex colours. These
controls affect terrain, props, shrubs and actors, including metallic actors.
They do not recolour the sky, HUD, menu model previews, or the separate water
and particle/effect vertex-colour passes. They adjust the level's vertex lighting rather than moving lights.

Fog strength runs from 0% to 200%. Fog distance runs from 0.25x to 4x; a larger
value places the fog farther away. Red, green and blue choose the fog colour.
The first colour edit starts from the current level colour. Resetting any fog
colour row restores all three colour channels to the level's changing colour.
The distance control changes rendered fog only: it does not extend object draw
distance, change detail levels or clipping, or affect physics. Special sea
effects keep their own fog behaviour. Menu previews keep their normal fog.

F4 preserves your lighting/fog edits while showing the original appearance.
F3 resets lighting and fog; use F8 in the sky picker to restore the sky too.
Opening one editor closes the other panel without undoing its settings.
Lighting and fog edits pause during title screens, movies, cutscenes and ship
travel, then return in ordinary level view. Changing levels resets all edits.
Settings are session-only; saved presets are not yet supported.

This mod contains no original game assets and does not edit your extracted data.

SWITCH BACK OR REMOVE

Close the game and start your usual ReRAC version. A standalone copy can be
removed by deleting its extracted mod folder; keep your game data separately.
If you installed through the launcher, return to Version Management and set
your usual version active. To remove this mod, use "Delete this installed
build" on the ReRAC @RUNTIME_VERSION@ entry. The launcher keeps your game data
and saves.

This preview ships its own runtime because ReRAC does not yet provide a
general plugin loader. It cannot be combined automatically with another
mod that also supplies a different runtime.

PREVIEW STATUS AND HELP

Code checks and automated tests do not replace gameplay testing. Visual
validation with extracted disc data is still required for this preview.
Share problems with the mod author in the current preview's pull request:
https://github.com/crystalfissure/rerac/pulls

Include the host planet, selected sky, lighting/fog values, steps to reproduce,
a screenshot if useful, and skybox-mod-last-run.log from the mod folder.
Launcher users can find their game log at ReRAC Launcher > Help > Logs folder.
If the game was started with RC_SKY_SWAPPER=0, remove that setting and restart
to enable the picker. Original/F8 restores the sky for the current session.
RC_ENVIRONMENT_MOD=0 disables the lighting/fog editor; remove that setting and
restart to enable it.

CREDITS AND LICENSE

ReRAC contributors provide the underlying runtime and extractor:
https://github.com/re-rac/rerac

This is an unofficial community mod, not an official ReRAC release. The
runtime and this mod are distributed under the ISC license in LICENSE.txt.
RUNTIME_README.txt contains the upstream runtime's additional usage notes.
