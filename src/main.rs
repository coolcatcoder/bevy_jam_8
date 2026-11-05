use avian2d::prelude::*;
pub use bevy::prelude;
use bevy::prelude::*;

use crate::selection::{Action, Select, Selectable, Selected, TargetFor};

mod input;
mod lion;
mod map;
mod moss;
mod objects;
mod puzzles;
mod save;
mod save_format;
mod selection;
mod ui;
mod water;

const DEBUG: bool = false;

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            selection::plugin,
            ui::plugin,
            input::plugin,
            objects::plugin,
            save::plugin,
            puzzles::plugin,
            PhysicsPlugins::default().with_length_unit(10.),
            //PhysicsDebugPlugin,
            moss::plugin,
            lion::plugin,
            water::plugin,
        ))
        .insert_resource(Gravity(Vec2::ZERO))
        .add_systems(Startup, (map, camera))
        .run()
}

fn map(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Sprite {
            image: asset_server.load("map.png"),
            ..default()
        },
        Transform::from_translation(Vec3::Z * -100.),
        TargetFor(vec![Action::Move, Action::DebugSpawn]),
    ));
}

pub fn camera(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut projection = OrthographicProjection::default_2d();
    projection.scale = 0.3;

    let _camera = commands
        .spawn((Camera2d, Msaa::Off, Projection::Orthographic(projection)))
        .id();

    // let player = commands
    //     .spawn((
    //         Sprite {
    //             image: asset_server.load("player/negative_x.png"),
    //             ..default()
    //         },
    //         Selectable {
    //             portrait: asset_server.load("player/portrait.png"),
    //             name: "James".into(),
    //             radius: 15.,
    //             actions: vec![Action::Move, Action::DebugSpawn],
    //         },
    //     ))
    //     .id();
    // An entity must be selected at all times. Start out with a dummy.
    let selection_dummy = commands
        .spawn((
            Selectable {
                portrait: asset_server.load("player/portrait.png"),
                name: "James".into(),
                radius: 15.,
                actions: vec![],
            },
            Transform::default(),
        ))
        .id();

    commands.insert_resource(Selected(selection_dummy));
    commands.trigger(Select);
}
