use avian2d::prelude::*;
use bevy::{
    prelude::*,
    sprite::{Anchor, SpriteAlphaMode},
};

use crate::{
    DEBUG,
    lion::Lion,
    moss::{CurrentWires, WireManager},
    selection::{Action, MoveTowards, PerformingAction, Select, Selectable, Selected, TargetFor},
    water::{Water, on_collision_end, on_collision_start},
};

pub fn plugin(app: &mut App) {
    app.add_observer(on_spawn_object)
        .add_systems(Startup, LoadedObjects::insert)
        .add_systems(Update, manage_depth);
}

const OBJECTS: &[&[&str]] = &[
    &["player/negative_x.png"],
    &["tree/tree.png"],
    &["wall.png"],
    &["statue.png", "statue_lit.png"],
    &["wire.png"],
    &["lion_head/positive_x.png", "lion_head/positive_x_lit.png"],
    &["boulder.png"],
    &["water.png"],
    &["lily.png"],
    &[if DEBUG {
        "bound.png"
    } else {
        "bound_empty.png"
    }],
];

#[derive(Resource)]
pub struct LoadedObjects {
    objects: Vec<Vec<Handle<Image>>>,
}

impl LoadedObjects {
    fn insert(mut commands: Commands, asset_server: Res<AssetServer>) {
        commands.insert_resource(LoadedObjects {
            objects: OBJECTS
                .iter()
                .map(|states| {
                    states
                        .iter()
                        .map(|state| asset_server.load(*state))
                        .collect()
                })
                .collect(),
        });
    }

    pub fn debug_name(&self, id: u8, state: u8) -> &'static str {
        OBJECTS[id as usize][state as usize]
    }

    pub fn length(&self) -> u8 {
        self.objects.len() as u8
    }

    pub fn states(&self, id: u8) -> u8 {
        self.objects[id as usize].len() as u8
    }

    pub fn update(&self, object: &Object, sprite: &mut Sprite) {
        sprite.image = self.objects[object.id as usize][object.state as usize].clone();
    }
}

#[derive(Component)]
pub struct Object {
    pub id: u8,
    pub state: u8,
    pub ghost: bool,
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
    mut current_wires: ResMut<CurrentWires>,
    objects: Query<(Entity, &Object)>,
    performing_action: Res<PerformingAction>,
    mut wire_managers: Query<&mut WireManager>,
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
            ghost: on.ghost,
        },
        ManageDepth,
        Anchor::BOTTOM_CENTER,
    ));

    match on.id {
        0 => {
            let mut actions = vec![Action::Move, Action::Grow, Action::Transmute];
            if DEBUG {
                actions.extend([
                    Action::DebugSpawn,
                    Action::DebugPrintTransform,
                    Action::ChangeState,
                    Action::DeleteObject,
                    Action::PuzzleElement,
                ]);
            }
            entity.insert((
                Selectable {
                    portrait: asset_server.load("player/portrait.png"),
                    name: "James".into(),
                    radius: 15.,
                    actions,
                },
                TargetFor::new([]),
            ));

            if on.ghost {
                return;
            }

            entity
                .insert((
                    RigidBody::Dynamic,
                    MoveTowards::None,
                    LockedAxes::ROTATION_LOCKED,
                ))
                .with_child((Collider::circle(5.), Transform::from_xyz(0., 5., 0.)));

            selected.0 = entity.id();
            commands.trigger(Select);
        }
        1 => {
            entity.insert(TargetFor::new([Action::Transmute]));

            if on.ghost {
                return;
            }

            entity
                .insert((
                    RigidBody::Dynamic,
                    LockedAxes::ALL_LOCKED,
                    LinearDamping(1.),
                    Mass(50.),
                    Dominance(1),
                ))
                .with_child((Collider::circle(3.), Transform::from_xyz(10., 3., 0.)));
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
        3 => {
            entity.insert((TargetFor::new([Action::Grow]),));

            if on.ghost {
                return;
            }

            entity
                .insert(RigidBody::Static)
                .with_child((Collider::circle(10.), Transform::from_xyz(0., 10., 0.)));
        }
        4 => {
            entity.insert((TargetFor::new([]),));

            if !matches!(performing_action.0, Some(Action::DebugSpawn)) {
                return;
            }
            if on.ghost {
                info!("Updating Moss");
                current_wires.ghost = Some(entity.id());
                current_wires.wire_manager = None;
                return;
            }

            if current_wires.ghost.is_none() {
                for (object_entity, object) in objects {
                    if object.ghost {
                        current_wires.ghost = Some(object_entity);
                        current_wires.wire_manager = None;
                    }
                }
            }

            let entity = entity.id();
            match current_wires.wire_manager {
                Some(wire_manager) => {
                    let mut wire_manager = wire_managers.get_mut(wire_manager).else_error()?;
                    wire_manager.0.push(entity);
                }
                None => {
                    current_wires.wire_manager =
                        Some(commands.spawn(WireManager(vec![entity])).id());
                }
            }
        }
        5 => {
            entity.insert((TargetFor::new([Action::Grow]), Lion(false, default())));

            if on.ghost {
                return;
            }

            entity
                .insert((
                    RigidBody::Static,
                    CollisionEventsEnabled,
                    Collider::circle(100.),
                    Sensor,
                ))
                .with_child((Collider::circle(10.), Transform::from_xyz(0., 10., 0.)));
        }
        6 => {
            entity.insert(TargetFor::new([Action::Transmute]));

            if on.ghost {
                return;
            }

            entity
                .insert((
                    RigidBody::Dynamic,
                    LockedAxes::ALL_LOCKED,
                    LinearDamping(1.),
                    Mass(50.),
                    Dominance(1),
                ))
                .with_child((
                    Collider::rectangle(25., 10.),
                    Transform::from_xyz(0., 10., 0.),
                ));
        }
        7 => {
            entity.insert(TargetFor::new([]));

            if on.ghost {
                return;
            }

            let entity = entity.id();
            let solid = commands.spawn_empty().id();

            commands
                .entity(entity)
                .insert((RigidBody::Static, Water(default(), solid)));
            commands.entity(solid).insert((
                ChildOf(entity),
                Collider::circle(6.),
                Transform::from_xyz(0., 6., 0.),
            ));

            commands
                .spawn((
                    ChildOf(entity),
                    Collider::circle(8.),
                    Transform::from_xyz(0., 6., 0.),
                    CollisionEventsEnabled,
                    Sensor,
                ))
                .observe(on_collision_start)
                .observe(on_collision_end);
        }
        8 => {
            entity.insert(TargetFor::new([Action::Transmute]));

            if on.ghost {
                return;
            }

            // TODO: No mass/inertia? Why am I getting that warning?
            entity
                .insert((
                    RigidBody::Dynamic,
                    LockedAxes::ALL_LOCKED,
                    LinearDamping(1.),
                    Mass(50.),
                    AngularInertia(0.),
                    Dominance(1),
                ))
                .with_child((
                    Collider::rectangle(20., 15.),
                    Transform::from_xyz(0., 5., 0.),
                    Sensor,
                ));
        }
        9 => {
            entity.insert(TargetFor::new([]));

            if on.ghost {
                return;
            }

            entity
                .insert(RigidBody::Static)
                .with_child((Collider::circle(20.), Transform::from_xyz(0., 20., 0.)));
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
