use bevy::{
    camera::primitives::Aabb,
    color::palettes::css::WHITE,
    input_management::{ActionTemplate, DPad, Input, Wasd},
    prelude::*,
};

use crate::{
    DEBUG,
    objects::{LoadedObjects, Object},
    puzzles::PuzzleElements,
    selection::DebugSpawn,
};

pub fn plugin(app: &mut App) {
    app.add_plugins(bevy::input_management::plugin)
        .add_systems(Startup, bindings)
        .add_systems(
            Update,
            (
                camera_move,
                cycle_debug_spawn,
                overlay_aabb,
                overlay_puzzle_element,
            ),
        );
}

fn bindings(mut input: ResMut<Input>) {
    input.bind::<CameraMove>(Wasd);
    input.bind::<CameraMove>(DPad);

    if DEBUG {
        input.bind::<OverlayAabb>(KeyCode::Numpad7);
        input.bind::<OverlayPuzzleElement>(KeyCode::Numpad8);
        input.bind::<IncreasePuzzleElement>(KeyCode::Equal);
        input.bind::<DecreasePuzzleElement>(KeyCode::Minus);
        input.bind::<CycleDebugSpawn>(KeyCode::Enter);
        input.bind::<SaveObjects>(KeyCode::F1);
    }
}

struct CameraMove;
impl ActionTemplate for CameraMove {
    type Template = Vec2;
}

fn camera_move(
    input: Res<Input>,
    mut camera: Single<&mut Transform, With<Camera>>,
    time: Res<Time>,
) {
    const SPEED: f32 = 200.;

    let direction = input.held::<CameraMove>();
    if direction == Vec2::ZERO {
        return;
    }

    let velocity = direction * SPEED;
    let displacement = velocity * time.delta_secs();
    camera.translation.x += displacement.x;
    camera.translation.y += displacement.y;
}

struct CycleDebugSpawn;
impl ActionTemplate for CycleDebugSpawn {
    type Template = bool;
}

fn cycle_debug_spawn(
    mut debug_spawn: ResMut<DebugSpawn>,
    input: Res<Input>,
    loaded_objects: Res<LoadedObjects>,
) {
    if !input.pressed::<CycleDebugSpawn>() {
        return;
    }

    debug_spawn.0 += 1;
    debug_spawn.0 %= loaded_objects.length();

    info!(
        "debug_spawn: {}",
        loaded_objects.debug_name(debug_spawn.0, 0)
    );
}

pub struct SaveObjects;
impl ActionTemplate for SaveObjects {
    type Template = bool;
}

pub struct IncreasePuzzleElement;
impl ActionTemplate for IncreasePuzzleElement {
    type Template = bool;
}
pub struct DecreasePuzzleElement;
impl ActionTemplate for DecreasePuzzleElement {
    type Template = bool;
}

pub struct OverlayAabb;
impl ActionTemplate for OverlayAabb {
    type Template = bool;
}
fn overlay_aabb(
    input: Res<Input>,
    aabbs: Query<(&Transform, &Aabb), With<Object>>,
    mut enabled: Local<bool>,
    mut gizmos: Gizmos,
) {
    if input.pressed::<OverlayAabb>() {
        enabled.toggle();
    }
    if !*enabled {
        return;
    }

    for (transform, aabb) in aabbs {
        gizmos.rect_2d(
            aabb.center.xy() + transform.translation.xy(),
            aabb.half_extents.xy() * 2.,
            WHITE,
        );
    }
}

pub struct OverlayPuzzleElement;
impl ActionTemplate for OverlayPuzzleElement {
    type Template = bool;
}
fn overlay_puzzle_element(
    input: Res<Input>,
    aabbs: Query<(&Transform, &Aabb), With<Object>>,
    mut enabled: Local<bool>,
    mut gizmos: Gizmos,
    puzzle_elements: Res<PuzzleElements>,
) {
    if input.pressed::<OverlayPuzzleElement>() {
        enabled.toggle();
    }
    if !*enabled {
        return;
    }

    for (entity, element) in puzzle_elements.entity_to_element.iter() {
        let (transform, aabb) = aabbs.get(*entity).else_error()?;
        gizmos.text_2d(
            aabb.center.xy() + transform.translation.xy(),
            &format!("e:{element}"),
            5.,
            Vec2::ZERO,
            WHITE,
        );
    }
}
