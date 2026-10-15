//! Live planet-sky picker. Base disc data is never changed. F6 opens the panel,
//! Page Up/Down chooses a source, F7 applies it, and F8 restores the original.
//! A single background loader owns donor decompression; GPU creation stays on the main thread.

use anyhow::{bail, Context, Result};
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use std::path::Path;
use std::sync::Mutex;
use std::thread::JoinHandle;

use crate::sky_render::LevelSky;

pub const PLANETS: [&str; 19] = [
    "Veldin (opening)", "Novalis", "Aridia", "Kerwan", "Eudora", "Rilgar",
    "Blarg Station", "Umbris", "Batalia", "Gaspar", "Orxon", "Pokitaru",
    "Hoven", "Gemlik Base", "Oltanis", "Quartu", "Kalebo III", "Drek's Fleet",
    "Veldin (finale)",
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Selection {
    #[default]
    Original,
    Planet(u32),
}

impl Selection {
    fn parse(s: &str) -> Result<Self> {
        if s.trim().eq_ignore_ascii_case("original") { return Ok(Self::Original); }
        let id: u32 = s.trim().parse().context("sky source must be 'original' or a level number 0..18")?;
        if id >= PLANETS.len() as u32 { bail!("sky source must be a level number 0..18"); }
        Ok(Self::Planet(id))
    }

    fn step(self, direction: i32) -> Self {
        let index = match self { Self::Original => 0, Self::Planet(id) => id as i32 + 1 };
        match (index + direction).rem_euclid(PLANETS.len() as i32 + 1) {
            0 => Self::Original,
            n => Self::Planet(n as u32 - 1),
        }
    }

    fn label(self) -> &'static str {
        match self { Self::Original => "Original", Self::Planet(id) => PLANETS[id as usize] }
    }
}

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct SkyStartup;
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct SkyPostStartup;
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct SkySwapSet;

/// Only the level's sky shells and stars; never the title/flight sky or world meshes.
#[derive(Component)]
pub(crate) struct SkyOwned;
#[derive(Component)]
struct PickerPanel;
#[derive(Component)]
struct PickerText;

struct Pending {
    generation: u64,
    source: u32,
    worker: Mutex<Option<JoinHandle<Result<LevelSky>>>>,
}

#[derive(Resource, Default)]
struct Picker {
    open: bool,
    selected: Selection,
    active: Selection,
    queued: Option<Selection>,
    pending: Option<Pending>,
    /// The host sky is retained only while a donor is active.
    original: Option<LevelSky>,
    status: String,
}

pub struct SkySwapPlugin;

impl Plugin for SkySwapPlugin {
    fn build(&self, app: &mut App) {
        // Baseline captures must stay unmodified unless the tester explicitly opts in.
        let setting = std::env::var("RC_SKY_SWAPPER").unwrap_or_default();
        if setting == "0" || (crate::determinism::deterministic() && setting != "1") { return; }
        let mut picker = Picker::default();
        if let Ok(source) = std::env::var("RC_SKY_SOURCE") {
            match Selection::parse(&source) {
                Ok(s) => { picker.selected = s; picker.queued = Some(s); }
                Err(e) => { eprintln!("sky picker: {e}"); picker.status = e.to_string(); }
            }
        }
        app.insert_resource(picker)
            .add_systems(Startup, setup_panel)
            .add_systems(Update, update.in_set(SkySwapSet))
            .add_systems(crate::level_switch::LevelUnload, on_level_unload);
        println!("sky picker: F6 panel | Page Up/Down source | F7 apply | F8 original");
    }
}

/// Donor-only load using a short-lived in-memory lump store. Browsing does not retain every
/// donor's entire decompressed level in the global cache, nor evict data a live level needs.
fn load_donor(root: &Path, source: u32) -> Result<LevelSky> {
    if source >= PLANETS.len() as u32 { bail!("invalid sky source {source}"); }
    let index = crate::disc_source::level_file(root, source, "core_index.bin")?;
    let store = rc_data::Lumps::with_cache_dir(None);
    let data = store.get(root, &format!("levels/{source:02}/core_data.bin"))?;
    let core = rc_formats::level::parse_level_core(&index, data.len())?;
    let mut sky = crate::sky_render::load(&core, &data, source)?
        .context("the selected level has no sky")?;
    if sky.sky.shells.is_empty() { bail!("the selected sky has no shells"); }
    // Reject an incomplete donor before replacing anything; don't silently draw white textures.
    for shell in &sky.sky.shells {
        if !shell.textured() { continue; }
        for cluster in &shell.clusters {
            for vertex in rc_formats::sky::sky_gs_vertices(shell, cluster) {
                if !sky.textures.iter().any(|t| t.index == vertex.texture as usize) {
                    bail!("sky texture {} is missing", vertex.texture);
                }
            }
        }
    }
    sky.star_tables = Some(crate::sky_stars::load_tables_from(root, source)?);
    Ok(sky)
}

fn world_available(world: &World) -> bool {
    !crate::saves::front_end_active()
        && !world.get_resource::<crate::travel_render::Travel>()
            .is_some_and(|t| t.blanked() || t.mode.active() || t.transition_fog().is_some())
        && !world.get_resource::<crate::movie_render::MovieState>().is_some_and(|m| m.busy())
        && !world.get_resource::<crate::scene_render::ActiveScene>().is_some_and(|s| s.running)
}

fn setup_panel(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: px(12), left: px(12), max_width: percent(94),
            padding: UiRect::all(px(12)), ..default()
        },
        BackgroundColor(Color::srgba(0.025, 0.035, 0.065, 0.94)),
        GlobalZIndex(i32::MAX), Visibility::Hidden,
        PickerPanel, crate::level_switch::KeepAcrossLevels,
        Name::new("sky picker"),
    )).with_children(|p| {
        p.spawn((Text::new(""), TextFont { font_size: bevy::text::FontSize::Px(16.0), ..default() },
            TextColor(Color::WHITE), PickerText));
    });
}

fn on_level_unload(mut picker: ResMut<Picker>) {
    picker.original = None;
    picker.selected = Selection::Original;
    picker.active = Selection::Original;
    picker.queued = None;
    picker.status = "New level: original sky restored.".into();
    // Keep the one worker slot occupied until it exits. Its generation makes its result stale.
}

fn update(world: &mut World) {
    let available = world_available(world);
    update_picker(world, available);
}

fn update_picker(world: &mut World, available: bool) {
    let generation = world.resource::<crate::level_switch::LevelGeneration>().0;
    let keys = world.resource::<ButtonInput<KeyCode>>();
    let (toggle, previous, next, apply, restore) = (
        keys.just_pressed(KeyCode::F6), keys.just_pressed(KeyCode::PageUp),
        keys.just_pressed(KeyCode::PageDown), keys.just_pressed(KeyCode::F7),
        keys.just_pressed(KeyCode::F8),
    );
    world.resource_scope(|world, mut p: Mut<Picker>| {
        if toggle { p.open = !p.open; }
        if available && p.open {
            if previous { p.selected = p.selected.step(-1); }
            if next { p.selected = p.selected.step(1); }
            if apply { p.queued = Some(p.selected); }
            if restore { p.queued = Some(Selection::Original); }
        }
        // A new request supersedes a finished but unapplied old one. Only one worker may run.
        if available && p.queued == Some(Selection::Original) {
            p.queued = None;
            if let Some(job) = &mut p.pending { job.generation = u64::MAX; }
            restore_sky(world, &mut p);
        }
        let finished = p.pending.as_ref().is_some_and(|job|
            job.worker.lock().unwrap_or_else(|e| e.into_inner()).as_ref().is_some_and(|w| w.is_finished()));
        if finished && (available || p.pending.as_ref().is_some_and(|j| j.generation != generation)) {
            let mut job = p.pending.take().unwrap();
            let result = job.worker.get_mut().unwrap_or_else(|e| e.into_inner()).take().unwrap().join()
                .unwrap_or_else(|_| Err(anyhow::anyhow!("sky loader panicked")));
            if job.generation == generation && p.queued.is_none() {
                match result {
                    Ok(sky) => {
                        if world.resource::<crate::Level>().0.sky.is_none() {
                            p.status = "This level has no original sky to restore; swap skipped.".into();
                        } else {
                            replace_sky(world, &mut p, sky);
                            p.active = Selection::Planet(job.source);
                            p.status = format!("Applied {}.", p.active.label());
                            println!("sky picker: applied {} ({:02})", p.active.label(), job.source);
                        }
                    }
                    Err(e) => {
                        eprintln!("sky picker: {e:#}");
                        p.status = format!("Could not load sky: {e}. Existing sky kept.");
                    }
                }
            }
        }
        if available && p.pending.is_none() {
            if let Some(selection) = p.queued.take() {
                match selection {
                    Selection::Original => restore_sky(world, &mut p),
                    Selection::Planet(source) if source == crate::level_load::level_index() => restore_sky(world, &mut p),
                    Selection::Planet(source) => {
                        let root = crate::level_load::extracted_root();
                        match std::thread::Builder::new().name("sky donor load".into()).spawn(move || load_donor(&root, source)) {
                            Ok(worker) => {
                                p.pending = Some(Pending { source, generation, worker: Mutex::new(Some(worker)) });
                                p.status = format!("Loading {}...", selection.label());
                            }
                            Err(e) => p.status = format!("Cannot start sky loader: {e}"),
                        }
                    }
                }
            }
        }
        let text = format!(
            "SKY PICKER\nCurrent: {}\nSelected: {}\n\nPage Up / Page Down: choose\nF7: apply   F8: restore   F6: close\n\n{}",
            p.active.label(), p.selected.label(),
            if available { p.status.as_str() } else { "Waiting for normal level view..." },
        );
        for mut t in world.query_filtered::<&mut Text, With<PickerText>>().iter_mut(world) {
            if t.0 != text { t.0.clone_from(&text); }
        }
        for mut v in world.query_filtered::<&mut Visibility, With<PickerPanel>>().iter_mut(world) {
            let wanted = if p.open && available { Visibility::Visible } else { Visibility::Hidden };
            if *v != wanted { *v = wanted; }
        }
    });
}

fn clear_draws(world: &mut World) {
    let entities: Vec<Entity> = world.query_filtered::<Entity, With<SkyOwned>>().iter(world).collect();
    for entity in entities { world.despawn(entity); }
    crate::sky_render::reset_sky_state(world);
    world.remove_resource::<crate::sky_stars::StarSim>();
    // Bevy releases GPU assets as their last strong handle is dropped with these entities/resources.
}

fn rebuild(world: &mut World) {
    world.run_schedule(SkyStartup);
    world.run_schedule(SkyPostStartup);
}

fn replace_sky(world: &mut World, picker: &mut Picker, sky: LevelSky) {
    if picker.original.is_none() {
        picker.original = world.resource_mut::<crate::Level>().0.sky.take();
        crate::sky_stars::preserve_original(world);
    }
    clear_draws(world);
    world.resource_mut::<crate::Level>().0.sky = Some(sky);
    rebuild(world);
}

fn restore_sky(world: &mut World, picker: &mut Picker) {
    if let Some(sky) = picker.original.take() {
        clear_draws(world);
        world.resource_mut::<crate::Level>().0.sky = Some(sky);
        rebuild(world);
        crate::sky_stars::restore_original(world);
    }
    picker.active = Selection::Original;
    picker.status = "Original sky restored.".into();
    println!("sky picker: original sky restored");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn failed_job(generation: u64) -> Pending {
        let worker = std::thread::spawn(|| Err(anyhow::anyhow!("synthetic missing donor")));
        while !worker.is_finished() { std::thread::yield_now(); }
        Pending { generation, source: 13, worker: Mutex::new(Some(worker)) }
    }

    fn picker_world(picker: Picker) -> World {
        let mut world = World::new();
        world.insert_resource(picker);
        world.insert_resource(ButtonInput::<KeyCode>::default());
        world.insert_resource(crate::level_switch::LevelGeneration(5));
        world
    }

    #[test]
    fn stale_load_cannot_change_new_level_status_or_selection() {
        let mut world = picker_world(Picker {
            pending: Some(failed_job(4)), status: "New level".into(), ..default()
        });
        // Even while travel is still blocking the picker, obsolete results are drained.
        update_picker(&mut world, false);
        let p = world.resource::<Picker>();
        assert!(p.pending.is_none());
        assert_eq!(p.active, Selection::Original);
        assert_eq!(p.status, "New level");
    }

    #[test]
    fn failed_load_keeps_active_sky_and_restore_cancels_finished_request() {
        let mut world = picker_world(Picker {
            active: Selection::Planet(3), pending: Some(failed_job(5)), ..default()
        });
        update_picker(&mut world, true);
        assert_eq!(world.resource::<Picker>().active, Selection::Planet(3));
        assert!(world.resource::<Picker>().status.contains("Existing sky kept"));
        {
            let mut p = world.resource_mut::<Picker>();
            p.pending = Some(failed_job(5));
            p.queued = Some(Selection::Original);
        }
        update_picker(&mut world, true);
        let p = world.resource::<Picker>();
        assert!(p.pending.is_none());
        assert_eq!(p.active, Selection::Original);
        assert_eq!(p.status, "Original sky restored.");
    }

    #[test]
    fn selection_rejects_invalid_sources_and_wraps() {
        for s in ["-1", "19", "4294967295", "novalis", ""] { assert!(Selection::parse(s).is_err()); }
        assert_eq!(Selection::parse(" 18 ").unwrap(), Selection::Planet(18));
        assert_eq!(Selection::parse("Original").unwrap(), Selection::Original);
        assert_eq!(Selection::Original.step(-1), Selection::Planet(18));
        assert_eq!(Selection::Planet(18).step(1), Selection::Original);
    }

    #[test]
    fn cleanup_keeps_world_and_camera_entities() {
        let mut world = World::new();
        let shell = world.spawn(SkyOwned).id();
        let star = world.spawn(SkyOwned).id();
        let terrain = world.spawn_empty().id();
        let camera = world.spawn(crate::sky_render::SkyCamera).id();
        world.insert_resource(crate::sky_render::SkyStarOrder(3));
        clear_draws(&mut world);
        assert!(world.get_entity(shell).is_err());
        assert!(world.get_entity(star).is_err());
        assert!(world.get_entity(terrain).is_ok());
        assert!(world.get_entity(camera).is_ok());
        assert!(!world.contains_resource::<crate::sky_render::SkyStarOrder>());
    }
}
