//! Session-only vertex-colour and fog controls. The original simulation, assets and sky stay intact.
//! F5 opens the editor; every edit is relative to the current level's native appearance.

use bevy::prelude::*;
use crate::game_camera::FogOverride;

#[derive(Resource, Clone, Debug, PartialEq)]
pub struct EnvironmentSettings {
    pub brightness: f32,
    pub tint: Vec3,
    pub lift: f32,
    pub fog: FogOverride,
    /// Set immediately before fog/render preparation, never while in non-world views.
    pub active: bool,
}

impl Default for EnvironmentSettings {
    fn default() -> Self {
        Self { brightness: 1.0, tint: Vec3::ONE, lift: 0.0, fog: FogOverride::default(), active: false }
    }
}

fn bounded(value: f32, low: f32, high: f32, fallback: f32) -> f32 {
    if value.is_finite() { value.clamp(low, high) } else { fallback }
}

impl EnvironmentSettings {
    /// Shared buffer slots at bytes 48 and 64. `w = 0` explicitly bypasses the shader operation,
    /// preserving native vertex colours bit for bit when disabled or neutral.
    pub fn light_uniforms(&self) -> (Vec4, Vec4) {
        let identity = (Vec3::ONE.extend(0.0), Vec4::ZERO);
        if !self.active { return identity; }
        let brightness = bounded(self.brightness, 0.0, 2.0, 1.0);
        let tint = Vec3::new(
            bounded(self.tint.x, 0.0, 2.0, 1.0),
            bounded(self.tint.y, 0.0, 2.0, 1.0),
            bounded(self.tint.z, 0.0, 2.0, 1.0),
        );
        let mul = brightness * tint;
        let lift = bounded(self.lift, 0.0, 1.0, 0.0);
        if mul == Vec3::ONE && lift == 0.0 { return identity; }
        (mul.extend(1.0), Vec3::splat(lift).extend(0.0))
    }

    fn adjust(&mut self, row: Row, direction: i32, native_fog: [u8; 3]) {
        let amount = direction.signum() as f32;
        match row {
            Row::Brightness => self.brightness = bounded(self.brightness + amount * 0.05, 0.0, 2.0, 1.0),
            Row::Red | Row::Green | Row::Blue => {
                let i = row as usize - Row::Red as usize;
                self.tint[i] = bounded(self.tint[i] + amount * 0.05, 0.0, 2.0, 1.0);
            }
            Row::Lift => self.lift = bounded(self.lift + amount * 0.025, 0.0, 1.0, 0.0),
            Row::FogStrength => self.fog.strength = bounded(self.fog.strength + amount * 0.05, 0.0, 2.0, 1.0),
            Row::FogDistance => self.fog.distance = bounded(self.fog.distance + amount * 0.25, 0.25, 4.0, 1.0),
            Row::FogRed | Row::FogGreen | Row::FogBlue => {
                let i = row as usize - Row::FogRed as usize;
                let color = self.fog.color.get_or_insert(native_fog);
                color[i] = (color[i] as i32 + direction.signum() * 8).clamp(0, 255) as u8;
            }
        }
    }

    fn reset(&mut self, row: Row) {
        match row {
            Row::Brightness => self.brightness = 1.0,
            Row::Red | Row::Green | Row::Blue => self.tint[row as usize - Row::Red as usize] = 1.0,
            Row::Lift => self.lift = 0.0,
            Row::FogStrength => self.fog.strength = 1.0,
            Row::FogDistance => self.fog.distance = 1.0,
            Row::FogRed | Row::FogGreen | Row::FogBlue => self.fog.color = None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(usize)]
enum Row {
    #[default]
    Brightness, Red, Green, Blue, Lift, FogStrength, FogDistance, FogRed, FogGreen, FogBlue,
}

impl Row {
    const ALL: [Self; 10] = [Self::Brightness, Self::Red, Self::Green, Self::Blue, Self::Lift,
        Self::FogStrength, Self::FogDistance, Self::FogRed, Self::FogGreen, Self::FogBlue];

    fn step(self, direction: i32) -> Self {
        Self::ALL[(self as i32 + direction).rem_euclid(Self::ALL.len() as i32) as usize]
    }

    fn label(self, settings: &EnvironmentSettings, native_fog: [u8; 3]) -> String {
        match self {
            Self::Brightness => format!("Brightness: {:.0}%", settings.brightness * 100.0),
            Self::Red => format!("Vertex red: {:.0}%", settings.tint.x * 100.0),
            Self::Green => format!("Vertex green: {:.0}%", settings.tint.y * 100.0),
            Self::Blue => format!("Vertex blue: {:.0}%", settings.tint.z * 100.0),
            Self::Lift => format!("Shadow lift: {:.1}%", settings.lift * 100.0),
            Self::FogStrength => format!("Fog strength: {:.0}%", settings.fog.strength * 100.0),
            Self::FogDistance => format!("Fog distance: {:.2}x", settings.fog.distance),
            Self::FogRed | Self::FogGreen | Self::FogBlue => {
                let i = self as usize - Self::FogRed as usize;
                let value = settings.fog.color.unwrap_or(native_fog)[i];
                let mode = if settings.fog.color.is_none() { "level" } else { "custom" };
                format!("Fog {}: {value} ({mode})", ["red", "green", "blue"][i])
            }
        }
    }
}

#[derive(Resource, Default)]
struct Editor {
    open: bool,
    selected: Row,
    compare_original: bool,
}

impl Editor {
    fn apply(&mut self, action: Action, settings: &mut EnvironmentSettings, native_fog: [u8; 3]) {
        match action {
            Action::Select(row) => self.selected = row,
            Action::Adjust(row, direction) => {
                self.selected = row;
                self.compare_original = false;
                settings.adjust(row, direction, native_fog);
            }
            Action::Reset(row) => {
                self.selected = row;
                self.compare_original = false;
                settings.reset(row);
            }
            Action::Compare => self.compare_original = !self.compare_original,
            Action::ResetAll => {
                *settings = EnvironmentSettings::default();
                self.compare_original = false;
            }
            Action::Close => self.open = false,
        }
    }
}

#[derive(Component)]
struct Panel;
#[derive(Component)]
struct StatusText;
#[derive(Component)]
struct RowText(Row);
#[derive(Component)]
struct RowHighlight(Row);
#[derive(Component, Clone, Copy)]
enum Action { Select(Row), Adjust(Row, i32), Reset(Row), Compare, ResetAll, Close }

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        let setting = std::env::var("RC_ENVIRONMENT_MOD").unwrap_or_default();
        if setting == "0" || (crate::determinism::deterministic() && setting != "1") { return; }
        app.init_resource::<EnvironmentSettings>()
            .init_resource::<Editor>()
            .add_systems(Startup, setup_panel)
            .add_systems(Update, update.after(crate::sky_swap::SkySwapSet))
            .add_systems(PostUpdate, update_active.before(crate::fog_state::FogSet))
            .add_systems(crate::level_switch::LevelUnload, on_level_unload);
        println!("environment editor: F5 panel | Home/End row | [/] adjust | F4 compare | F3 reset");
    }
}

/// Opening the sky picker closes this panel without undoing its live appearance settings.
pub(crate) fn close_panel(world: &mut World) {
    if let Some(mut editor) = world.get_resource_mut::<Editor>() { editor.open = false; }
}

fn setup_panel(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute, top: px(8), left: px(8),
            width: px(490), max_width: percent(96), padding: UiRect::all(px(10)),
            flex_direction: FlexDirection::Column, row_gap: px(3), ..default()
        },
        BackgroundColor(Color::srgba(0.025, 0.035, 0.065, 0.96)),
        GlobalZIndex(i32::MAX), Visibility::Hidden, Panel, crate::mod_ui::ModOverlay,
        crate::level_switch::KeepAcrossLevels, Name::new("environment editor"),
    )).with_children(|panel| {
        panel.spawn((Text::new("LEVEL LIGHTING + FOG"),
            TextFont { font_size: bevy::text::FontSize::Px(18.0), ..default() }, TextColor(Color::WHITE)));
        panel.spawn((Text::new(""), TextFont { font_size: bevy::text::FontSize::Px(13.0), ..default() },
            TextColor(Color::srgb(0.65, 0.9, 1.0)), StatusText));
        for row in Row::ALL {
            panel.spawn((Node { width: percent(100), min_height: px(25), column_gap: px(4),
                align_items: AlignItems::Center, ..default() },
                BackgroundColor(Color::NONE), RowHighlight(row),
            )).with_children(|line| {
                line.spawn((Button, Action::Select(row), Node { flex_grow: 1.0, ..default() }))
                    .with_children(|label| {
                        label.spawn((Text::new(""), TextFont { font_size: bevy::text::FontSize::Px(14.0), ..default() },
                            TextColor(Color::WHITE), RowText(row)));
                    });
                let (reset_label, reset_width) = if matches!(row, Row::FogRed | Row::FogGreen | Row::FogBlue) {
                    ("Level RGB", 72)
                } else { ("Reset", 48) };
                for (label, action, width) in [("-", Action::Adjust(row, -1), 28),
                    ("+", Action::Adjust(row, 1), 28), (reset_label, Action::Reset(row), reset_width)] {
                    line.spawn((Button, action, Node { width: px(width), height: px(24),
                        justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                        BackgroundColor(Color::srgb(0.12, 0.17, 0.25))))
                        .with_children(|button| {
                            button.spawn((Text::new(label),
                                TextFont { font_size: bevy::text::FontSize::Px(13.0), ..default() }, TextColor(Color::WHITE)));
                        });
                }
            });
        }
        panel.spawn((Node { column_gap: px(6), margin: UiRect::top(px(4)), ..default() },))
            .with_children(|bar| {
                for (label, action) in [("F4 Compare", Action::Compare), ("F3 Reset all", Action::ResetAll), ("F5 Close", Action::Close)] {
                    bar.spawn((Button, action, Node { padding: UiRect::axes(px(8), px(4)), ..default() },
                        BackgroundColor(Color::srgb(0.12, 0.17, 0.25))))
                        .with_children(|button| {
                            button.spawn((Text::new(label),
                                TextFont { font_size: bevy::text::FontSize::Px(13.0), ..default() }, TextColor(Color::WHITE)));
                        });
                }
            });
        panel.spawn((Text::new("Home/End: select row   [ / ]: adjust   Delete: reset row\nFog: Level RGB / Delete resets all three colour channels.\nEdits are live; gameplay continues. Esc releases cursor.\nSession only. Changing levels resets all. F6: sky picker."),
            TextFont { font_size: bevy::text::FontSize::Px(12.0), ..default() },
            TextColor(Color::srgb(0.8, 0.83, 0.9))));
    });
}

fn on_level_unload(mut settings: ResMut<EnvironmentSettings>, mut editor: ResMut<Editor>) {
    *settings = EnvironmentSettings::default();
    *editor = Editor::default();
}

fn update_active(world: &mut World) {
    let active = crate::sky_swap::world_available(world) && !world.resource::<Editor>().compare_original;
    let mut settings = world.resource_mut::<EnvironmentSettings>();
    if settings.active != active { settings.active = active; }
}

fn update(world: &mut World) {
    let available = crate::sky_swap::world_available(world);
    let native_fog = world.get_resource::<crate::game_camera::GameFog>().map(|f| f.fog.color).unwrap_or([0; 3]);
    let keys = world.resource::<ButtonInput<KeyCode>>();
    let (toggle, previous, next, down, up, reset, compare, reset_all) = (
        keys.just_pressed(KeyCode::F5), keys.just_pressed(KeyCode::Home), keys.just_pressed(KeyCode::End),
        keys.just_pressed(KeyCode::BracketLeft), keys.just_pressed(KeyCode::BracketRight),
        keys.just_pressed(KeyCode::Delete), keys.just_pressed(KeyCode::F4), keys.just_pressed(KeyCode::F3),
    );
    let actions: Vec<Action> = world.query_filtered::<(&Interaction, &Action), Changed<Interaction>>()
        .iter(world).filter_map(|(interaction, action)| (*interaction == Interaction::Pressed).then_some(*action)).collect();
    // Interactions only repaint controls whose hover/press state changed.
    for (interaction, action, mut color) in world.query_filtered::<(&Interaction, &Action, &mut BackgroundColor), Changed<Interaction>>().iter_mut(world) {
        if matches!(action, Action::Select(_)) { continue; }
        let wanted = BackgroundColor(match interaction {
            Interaction::Pressed => Color::srgb(0.18, 0.4, 0.52),
            Interaction::Hovered => Color::srgb(0.2, 0.28, 0.4),
            Interaction::None => Color::srgb(0.12, 0.17, 0.25),
        });
        if *color != wanted { *color = wanted; }
    }
    world.resource_scope(|world, mut editor: Mut<Editor>| {
        if toggle && available {
            editor.open = !editor.open;
            if editor.open { crate::sky_swap::close_panel(world); }
        }
        if editor.open && available {
            if previous { editor.selected = editor.selected.step(-1); }
            if next { editor.selected = editor.selected.step(1); }
            let mut settings = world.resource_mut::<EnvironmentSettings>();
            let selected = editor.selected;
            if down { editor.apply(Action::Adjust(selected, -1), &mut settings, native_fog); }
            if up { editor.apply(Action::Adjust(selected, 1), &mut settings, native_fog); }
            if reset { editor.apply(Action::Reset(selected), &mut settings, native_fog); }
            if compare { editor.apply(Action::Compare, &mut settings, native_fog); }
            if reset_all { editor.apply(Action::ResetAll, &mut settings, native_fog); }
            for action in actions { editor.apply(action, &mut settings, native_fog); }
        }
        let visible = editor.open && available;
        for mut v in world.query_filtered::<&mut Visibility, With<Panel>>().iter_mut(world) {
            let wanted = if visible { Visibility::Visible } else { Visibility::Hidden };
            if *v != wanted { *v = wanted; }
        }
        if !visible { return; }
        let settings = world.resource::<EnvironmentSettings>().clone();
        for (row, mut text) in world.query::<(&RowText, &mut Text)>().iter_mut(world) {
            let prefix = if row.0 == editor.selected { "> " } else { "  " };
            let wanted = format!("{prefix}{}", row.0.label(&settings, native_fog));
            if text.0 != wanted { text.0 = wanted; }
        }
        for (row, mut color) in world.query::<(&RowHighlight, &mut BackgroundColor)>().iter_mut(world) {
            let wanted = BackgroundColor(if row.0 == editor.selected { Color::srgb(0.09, 0.16, 0.23) } else { Color::NONE });
            if *color != wanted { *color = wanted; }
        }
        let status = if editor.compare_original { "ORIGINAL LIGHTING/FOG - F4 returns to edits" } else { "EDITED LIGHTING/FOG - F4 compares originals" };
        for mut text in world.query_filtered::<&mut Text, With<StatusText>>().iter_mut(world) {
            if text.0 != status { text.0 = status.to_owned(); }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn environment_neutral_and_inactive_bypass_vertex_math() {
        let identity = (Vec3::ONE.extend(0.0), Vec4::ZERO);
        let mut settings = EnvironmentSettings::default();
        assert_eq!(settings.light_uniforms(), identity);
        settings.active = true;
        assert_eq!(settings.light_uniforms(), identity);
        settings.brightness = 1.5;
        settings.tint = Vec3::new(1.0, 0.5, 0.25);
        settings.lift = 0.1;
        assert_eq!(settings.light_uniforms(), (Vec4::new(1.5, 0.75, 0.375, 1.0), Vec4::new(0.1, 0.1, 0.1, 0.0)));
        settings.active = false;
        assert_eq!(settings.light_uniforms(), identity);
    }

    #[test]
    fn environment_clamps_controls_and_rejects_nonfinite_uniform_values() {
        let mut settings = EnvironmentSettings::default();
        for _ in 0..100 {
            for row in Row::ALL { settings.adjust(row, 1, [250, 0, 128]); }
        }
        assert_eq!(settings.brightness, 2.0);
        assert_eq!(settings.tint, Vec3::splat(2.0));
        assert_eq!(settings.lift, 1.0);
        assert_eq!(settings.fog, FogOverride { strength: 2.0, distance: 4.0, color: Some([255; 3]) });
        for _ in 0..100 {
            for row in Row::ALL { settings.adjust(row, -1, [250, 0, 128]); }
        }
        assert_eq!(settings.brightness, 0.0);
        assert_eq!(settings.tint, Vec3::ZERO);
        assert_eq!(settings.lift, 0.0);
        assert_eq!(settings.fog, FogOverride { strength: 0.0, distance: 0.25, color: Some([0; 3]) });
        settings.active = true;
        settings.brightness = f32::NAN;
        settings.tint = Vec3::new(f32::INFINITY, -9.0, 900.0);
        settings.lift = f32::NEG_INFINITY;
        assert_eq!(settings.light_uniforms(), (Vec4::new(1.0, 0.0, 2.0, 1.0), Vec4::ZERO));
    }

    #[test]
    fn environment_fog_colour_seeds_current_level_then_restores_tracking() {
        let mut settings = EnvironmentSettings::default();
        settings.adjust(Row::FogGreen, 1, [30, 70, 110]);
        assert_eq!(settings.fog.color, Some([30, 78, 110]));
        settings.adjust(Row::FogBlue, -1, [100, 200, 255]);
        assert_eq!(settings.fog.color, Some([30, 78, 102]));
        settings.reset(Row::FogRed);
        assert_eq!(settings.fog.color, None);
        settings.adjust(Row::FogBlue, 1, [100, 200, 250]);
        assert_eq!(settings.fog.color, Some([100, 200, 255]));
    }

    #[test]
    fn environment_comparison_preserves_edits_and_reset_all_restores_defaults() {
        let mut editor = Editor { open: true, ..default() };
        let mut settings = EnvironmentSettings::default();
        editor.apply(Action::Adjust(Row::Red, -1), &mut settings, [0; 3]);
        let edited = settings.clone();
        editor.apply(Action::Compare, &mut settings, [0; 3]);
        assert!(editor.compare_original);
        assert_eq!(settings, edited);
        editor.apply(Action::Compare, &mut settings, [0; 3]);
        assert!(!editor.compare_original);
        assert_eq!(settings, edited);
        editor.apply(Action::ResetAll, &mut settings, [0; 3]);
        assert_eq!(settings, EnvironmentSettings::default());
        assert!(!editor.compare_original);
        assert!(editor.open);
    }

    #[test]
    fn environment_row_reset_leaves_comparison_and_keeps_other_edits() {
        let mut settings = EnvironmentSettings { brightness: 0.5, tint: Vec3::splat(0.75), ..default() };
        let mut editor = Editor { compare_original: true, ..default() };
        editor.apply(Action::Reset(Row::Brightness), &mut settings, [0; 3]);
        assert!(!editor.compare_original, "make the reset visible, like any other edit");
        assert_eq!(settings.brightness, 1.0);
        assert_eq!(settings.tint, Vec3::splat(0.75));
    }

    #[test]
    fn environment_level_unload_drops_every_override_and_comparison() {
        let mut app = App::new();
        app.insert_resource(EnvironmentSettings { brightness: 0.2, tint: Vec3::splat(0.4), lift: 0.1,
            fog: FogOverride { strength: 0.2, distance: 3.0, color: Some([1, 2, 3]) }, active: true })
            .insert_resource(Editor { open: true, selected: Row::FogBlue, compare_original: true })
            .add_systems(Update, on_level_unload);
        app.update();
        assert_eq!(*app.world().resource::<EnvironmentSettings>(), EnvironmentSettings::default());
        let editor = app.world().resource::<Editor>();
        assert!(!editor.open && !editor.compare_original);
        assert_eq!(editor.selected, Row::Brightness);
    }
}
