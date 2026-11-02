pub use bevy::prelude;
use bevy::prelude::*;

use crate::selection::{Action, Select, Selectable, Selected, TargetFor};

mod input;
mod map;
mod objects;
mod selection;
mod ui;

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            selection::plugin,
            ui::plugin,
            input::plugin,
            objects::plugin,
        ))
        .add_systems(Startup, (map, player))
        .run()
}

fn map(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Sprite {
            image: asset_server.load("map.png"),
            ..default()
        },
        Transform::from_translation(Vec3::Z * -1.),
        TargetFor(vec![Action::Move, Action::DebugSpawn]),
    ));
}

pub fn player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut projection = OrthographicProjection::default_2d();
    projection.scale = 0.3;

    let _camera = commands
        .spawn((Camera2d, Msaa::Off, Projection::Orthographic(projection)))
        .id();

    let player = commands
        .spawn((
            Sprite {
                image: asset_server.load("player/negative_x.png"),
                ..default()
            },
            Selectable {
                portrait: asset_server.load("player/portrait.png"),
                name: "James".into(),
                radius: 15.,
                actions: vec![Action::Move, Action::DebugSpawn],
            },
        ))
        .id();

    commands.insert_resource(Selected(player));
    commands.trigger(Select);
}
