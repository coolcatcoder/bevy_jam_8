use bevy::prelude::*;

use crate::selection::{Action, Selectable};

pub fn plugin(app: &mut App) {
    app.add_observer(on_spawn_object);
}

pub const OBJECTS: &[&[&str]] = &[&["player/negative_x.png"]];

#[derive(Component)]
pub struct Object {
    pub id: u8,
    pub state: u8,
}

#[derive(EntityEvent)]
pub struct SpawnObject {
    pub entity: Entity,
    pub id: u8,
    pub state: u8,
    pub position: Vec2,
}

fn on_spawn_object(on: On<SpawnObject>, mut commands: Commands, asset_server: Res<AssetServer>) {
    match on.id {
        0 => {
            commands.entity(on.entity).insert((
                Transform::from_translation(on.position.extend(0.)),
                Sprite {
                    image: asset_server.load(OBJECTS[on.id as usize][on.state as usize]),
                    ..default()
                },
                Selectable {
                    portrait: asset_server.load("player/portrait.png"),
                    name: "James".into(),
                    radius: 15.,
                    actions: vec![Action::Move, Action::DebugSpawn],
                },
                Object {
                    id: on.id,
                    state: on.state,
                },
            ));
        }
        _ => unreachable!(),
    }
}
