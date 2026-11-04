use avian2d::prelude::*;
use bevy::{
    prelude::*,
    sprite::{Anchor, SpriteAlphaMode},
};

use crate::selection::{Action, MoveTowards, Select, Selectable, Selected, TargetFor};

pub fn plugin(app: &mut App) {
    app.add_observer(on_spawn_object)
        .add_systems(Update, manage_depth);
}

pub const OBJECTS: &[&[&str]] = &[
    &["player/negative_x.png"],
    &["tree/tree.png"],
    &["wall.png"],
];

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
    pub ghost: bool,
}

fn on_spawn_object(
    on: On<SpawnObject>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut selected: ResMut<Selected>,
) {
    let mut entity = commands.entity(on.entity);
    entity.insert((
        Transform::from_translation(on.position.extend(0.)),
        Sprite {
            image: asset_server.load(OBJECTS[on.id as usize][on.state as usize]),
            alpha_mode: SpriteAlphaMode::Mask(0.5),
            ..default()
        },
        Object {
            id: on.id,
            state: on.state,
        },
        ManageDepth,
        Anchor::BOTTOM_CENTER,
    ));

    match on.id {
        0 => {
            entity.insert((
                Selectable {
                    portrait: asset_server.load("player/portrait.png"),
                    name: "James".into(),
                    radius: 15.,
                    actions: vec![
                        Action::Move,
                        Action::DebugSpawn,
                        Action::DebugPrintTransform,
                        Action::ChangeState,
                        Action::DeleteObject,
                    ],
                },
                TargetFor::new([]),
            ));

            if on.ghost {
                return;
            }

            entity
                .insert((
                    RigidBody::Dynamic,
                    MoveTowards(None),
                    LockedAxes::ROTATION_LOCKED,
                ))
                .with_child((Collider::circle(5.), Transform::from_xyz(0., 5., 0.)));

            selected.0 = entity.id();
            commands.trigger(Select);
        }
        1 => {
            entity.insert((TargetFor::new([]),));
        }
        2 => {
            entity.insert((TargetFor::new([]),));

            if on.ghost {
                return;
            }

            entity
                .insert(RigidBody::Static)
                .with_child((Collider::circle(10.), Transform::from_xyz(0., 10., 0.)));
        }
        _ => unreachable!(),
    }
}

#[derive(Component)]
struct ManageDepth;
fn manage_depth(transforms: Query<&mut Transform, (Changed<Transform>, With<ManageDepth>)>) {
    for mut transform in transforms {
        //let transform = transform.bypass_change_detection();
        transform.translation.z = -(transform.translation.y / 10.);
    }
}
