use bevy::{
    input_management::{ActionTemplate, DPad, Input, Wasd},
    prelude::*,
};

use crate::{objects::OBJECTS, selection::DebugSpawn};

pub fn plugin(app: &mut App) {
    app.add_plugins(bevy::input_management::plugin)
        .add_systems(Startup, bindings)
        .add_systems(Update, (camera_move, cycle_debug_spawn));
}

fn bindings(mut input: ResMut<Input>) {
    input.bind::<CameraMove>(Wasd);
    input.bind::<CameraMove>(DPad);

    input.bind::<CycleDebugSpawn>(KeyCode::Enter);
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

fn cycle_debug_spawn(mut debug_spawn: ResMut<DebugSpawn>, input: Res<Input>) {
    if !input.pressed::<CycleDebugSpawn>() {
        return;
    }

    debug_spawn.0 += 1;
    debug_spawn.0 %= OBJECTS.len() as u8;

    info!("debug_spawn: {}", OBJECTS[debug_spawn.0 as usize][0]);
}
