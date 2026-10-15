//! Mod controls belong to the window, not the offscreen game frame: Bevy's UI focus
//! system only accepts pointer input from a camera whose target is a window.

use bevy::prelude::*;
use bevy::ui::{UiTargetCamera, UiTransform, Val2};
use bevy::window::PrimaryWindow;

#[derive(Component)]
pub(crate) struct ModOverlay;

pub struct ModUiPlugin;

impl Plugin for ModUiPlugin {
    fn build(&self, app: &mut App) {
        // Startup has spawned both the present camera and the optional mod panels.
        app.add_systems(PostStartup, target_window)
            .add_systems(PostUpdate, fit_panels.before(bevy::ui::UiSystems::Layout));
    }
}

fn target_window(
    mut commands: Commands,
    cameras: Query<Entity, With<crate::display::PresentCamera>>,
    panels: Query<Entity, With<ModOverlay>>,
) {
    let Ok(camera) = cameras.single() else { return };
    for panel in &panels { commands.entity(panel).insert(UiTargetCamera(camera)); }
}

/// The UI transform acts about the node centre. Offset by half the lost size so
/// shrinking keeps the original top-left anchor and the buttons stay inside the window.
fn fitted_transform(size: Vec2, available: Vec2) -> UiTransform {
    if !size.is_finite() || size.min_element() <= 0.0 || !available.is_finite() {
        return UiTransform::IDENTITY;
    }
    let scale = (available.max(Vec2::ONE) / size).min_element().min(1.0);
    UiTransform {
        scale: Vec2::splat(scale),
        translation: Val2::percent((scale - 1.0) * 50.0, (scale - 1.0) * 50.0),
        ..default()
    }
}

fn fit_panels(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut panels: Query<(&ComputedNode, &mut UiTransform), With<ModOverlay>>,
) {
    let Ok(window) = windows.single() else { return };
    let available = Vec2::new(window.width(), window.height()) - Vec2::splat(24.0);
    for (node, mut transform) in &mut panels {
        // ComputedNode is the untransformed physical layout from the previous frame.
        let size = node.size() * node.inverse_scale_factor();
        let want = fitted_transform(size, available);
        if *transform != want { *transform = want; }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::Propagate;
    use bevy::camera::RenderTarget;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::input::touch::Touches;
    use bevy::math::DVec2;
    use bevy::ui::{ComputedUiTargetCamera, IsDefaultUiCamera, UiGlobalTransform, UiScale, UiStack};
    use bevy::window::WindowRef;

    fn propagate_target(world: &mut World, panel: Entity) {
        world.run_system_once(bevy::ui::update::propagate_ui_target_cameras).unwrap();
        let target = world.get::<Propagate<ComputedUiTargetCamera>>(panel).unwrap().0;
        world.entity_mut(panel).insert(target);
    }

    #[test]
    fn mod_ui_window_target_restores_real_bevy_mouse_interaction() {
        let mut world = World::new();
        let mut window = Window::default();
        window.set_physical_cursor_position(Some(DVec2::new(50.0, 50.0)));
        world.spawn((window, PrimaryWindow));
        let game = world.spawn((Camera::default(), RenderTarget::Image(Handle::<Image>::default().into()), IsDefaultUiCamera)).id();
        let present = world.spawn((Camera::default(), RenderTarget::Window(WindowRef::Primary), crate::display::PresentCamera)).id();
        let panel = world.spawn((Button, ModOverlay, ComputedNode { size: Vec2::splat(100.0), ..default() },
            UiGlobalTransform::from_translation(Vec2::splat(50.0)), InheritedVisibility::VISIBLE)).id();
        let unrelated = world.spawn(Node::default()).id();
        let mut mouse = ButtonInput::<MouseButton>::default();
        mouse.press(MouseButton::Left);
        world.insert_resource(mouse);
        world.insert_resource(Touches::default());
        world.insert_resource(UiScale::default());
        world.insert_resource(UiStack { partition: std::iter::once(0..1).collect(), uinodes: vec![panel] });

        // Reproduce 0.2: default UI camera draws to an image and focus ignores it.
        propagate_target(&mut world, panel);
        world.run_system_once(bevy::ui::ui_focus_system).unwrap();
        assert_eq!(*world.get::<Interaction>(panel).unwrap(), Interaction::None);

        world.run_system_once(target_window).unwrap();
        propagate_target(&mut world, panel);
        world.run_system_once(bevy::ui::ui_focus_system).unwrap();
        assert_eq!(world.get::<UiTargetCamera>(panel).unwrap().entity(), present);
        assert_eq!(*world.get::<Interaction>(panel).unwrap(), Interaction::Pressed);
        assert!(world.get::<UiTargetCamera>(unrelated).is_none());
        assert!(matches!(world.get::<RenderTarget>(game), Some(RenderTarget::Image(_))));
    }

    #[test]
    fn mod_ui_small_windows_fit_with_top_left_anchor_and_restore_full_size() {
        let size = Vec2::new(490.0, 460.0);
        for available in [Vec2::new(296.0, 216.0), Vec2::new(532.0, 392.0), Vec2::new(1000.0, 744.0)] {
            let transform = fitted_transform(size, available);
            let affine = transform.compute_affine(1.0, size, available);
            let top_left = size * 0.5 + affine.transform_point2(size * -0.5);
            let bottom_right = size * 0.5 + affine.transform_point2(size * 0.5);
            assert!(top_left.abs().max_element() < 0.001, "keep the panel anchored at its original margin");
            assert!(bottom_right.cmple(available + Vec2::splat(0.001)).all());
        }
        assert_eq!(fitted_transform(size, Vec2::new(1000.0, 744.0)), UiTransform::IDENTITY);
        assert_eq!(fitted_transform(Vec2::ZERO, Vec2::ONE), UiTransform::IDENTITY);
    }
}
