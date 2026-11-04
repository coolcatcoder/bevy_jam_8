use avian2d::prelude::*;
use bevy::{
    color::palettes::css::{BLACK, LIME, WHITE},
    prelude::*,
    window::{CursorIcon, SystemCursorIcon},
};

use crate::{
    objects::{OBJECTS, Object, SpawnObject},
    ui::SelectionAction,
};

pub fn plugin(app: &mut App) {
    app.add_observer(MoveTowards::perform_action)
        .init_resource::<PerformingAction>()
        .init_resource::<DebugSpawn>()
        .add_observer(DebugSpawn::perform_action)
        .add_observer(debug_print_transform)
        .init_resource::<Ghost>()
        .add_observer(change_state)
        .add_systems(
            Update,
            (
                temporary_gizmo_that_should_be_replaced_with_a_sprite_of_a_pre_rendered_torus,
                MoveTowards::motion,
                DebugSpawn::spawn_and_despawn_ghost,
                DebugSpawn::haunt_the_pointer,
                DebugSpawn::update_haunted_sprite,
            ),
        );
}

#[derive(Component)]
#[require(Pickable {
    should_block_lower: true,
    is_hoverable: true,
})]
#[component(on_add = Self::on_add)]
pub struct Selectable {
    pub portrait: Handle<Image>,
    pub name: String,
    pub radius: f32,
    pub actions: Vec<Action>,
}
impl Selectable {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        world.commands().entity(context.entity).observe(
            |on: On<PointerClick>,
             performing_action: Res<PerformingAction>,
             mut selected: ResMut<Selected>,
             mut commands: Commands| {
                if performing_action.0.is_some() {
                    return;
                }

                selected.0 = on.entity;
                commands.trigger(Select);
            },
        );
    }
}

#[derive(Resource)]
pub struct Selected(pub Entity);

#[derive(Event)]
pub struct Select;

fn temporary_gizmo_that_should_be_replaced_with_a_sprite_of_a_pre_rendered_torus(
    mut gizmos: Gizmos,
    selected: Res<Selected>,
    selectables: Query<(&Selectable, &Transform)>,
) {
    let (selectable, transform) = selectables.get(selected.0).else_error()?;
    gizmos.circle_2d(transform.translation.xy(), selectable.radius, LIME);
}

#[derive(Default, Resource)]
pub struct PerformingAction(pub Option<Action>);

#[derive(Clone, PartialEq, Eq)]
pub enum Action {
    Move,
    DebugSpawn,
    DebugPrintTransform,
    ChangeState,
    DeleteObject,
}
impl Action {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Move => "Move.",
            Self::DebugSpawn => "Debug spawn.",
            Self::DebugPrintTransform => "Debug print transform.",
            Self::ChangeState => "Change state.",
            Self::DeleteObject => "Delete object.",
        }
    }
}

fn change_state(
    on: On<PerformAction>,
    mut object: Query<(&mut Object, &mut Sprite)>,
    asset_server: Res<AssetServer>,
) {
    if !matches!(on.action, Action::ChangeState) {
        return;
    }

    let (mut object, mut sprite) = object.get_mut(on.target).else_error()?;

    object.state += 1;
    object.state %= OBJECTS[object.id as usize].len() as u8;

    sprite.image = asset_server.load(OBJECTS[object.id as usize][object.state as usize]);
    info!("object.state: {}", object.state);
}

#[derive(Component)]
pub struct MoveTowards(pub Option<Vec2>);
impl MoveTowards {
    fn perform_action(on: On<PerformAction>, mut move_towardss: Query<&mut MoveTowards>) {
        if !matches!(on.action, Action::Move) {
            return;
        }

        let mut move_towards = move_towardss.get_mut(on.performer).else_error()?;
        move_towards.0 = Some(on.position);
    }

    fn motion(
        move_towardss: Query<(&mut MoveTowards, &Transform, &mut LinearVelocity)>,
        time: Res<Time>,
    ) {
        let time = time.delta_secs();
        for (mut move_towards, transform, mut linear_velocity) in move_towardss {
            const DISTANCE: f32 = 2.;

            match move_towards.0 {
                Some(target) => {
                    let direction = (target - transform.translation.xy()).normalize_or_zero();
                    let displacement = direction * DISTANCE;

                    linear_velocity.0 = displacement / time;

                    if transform.translation.xy().distance_squared(target) < (5. * 5.) {
                        move_towards.0 = None;
                    }
                }
                None => {
                    linear_velocity.0 = Vec2::ZERO;
                }
            }
        }
    }
}

#[derive(Resource, Default)]
struct Ghost(Option<Entity>);

#[derive(Resource, Default)]
pub struct DebugSpawn(pub u8);
impl DebugSpawn {
    fn perform_action(
        on: On<PerformAction>,
        mut commands: Commands,
        debug_spawn: Res<DebugSpawn>,
        mut performing_action: ResMut<PerformingAction>,
    ) {
        if !matches!(on.action, Action::DebugSpawn) {
            return;
        }

        let entity = commands.spawn_empty().id();
        commands.trigger(SpawnObject {
            entity,
            id: debug_spawn.0,
            state: 0,
            position: on.position,
            ghost: false,
        });

        performing_action.0 = Some(Action::DebugSpawn);
    }

    fn spawn_and_despawn_ghost(
        mut ghost: ResMut<Ghost>,
        performing_action: Res<PerformingAction>,
        mut commands: Commands,
        debug_spawn: Res<DebugSpawn>,
    ) {
        match (ghost.0, &performing_action.0) {
            (None, Some(Action::DebugSpawn)) => {
                let entity = commands.spawn_empty().id();
                commands.trigger(SpawnObject {
                    entity,
                    id: debug_spawn.0,
                    state: 0,
                    position: Vec2::ZERO,
                    ghost: true,
                });
                ghost.0 = Some(entity);
            }
            (Some(_), Some(Action::DebugSpawn)) => (),
            (Some(entity), _) => {
                commands.entity(entity).despawn();
                ghost.0 = None;
            }
            _ => (),
        }
    }

    fn haunt_the_pointer(
        ghost: Res<Ghost>,
        window: Single<&Window>,
        camera: Single<(&Camera, &GlobalTransform)>,
        mut transforms: Query<&mut Transform>,
    ) {
        let ghost = ghost.0.else_return()?;
        let mut ghost_transform = transforms.get_mut(ghost).else_return()?;
        let (camera, camera_transform) = *camera;

        let cursor_position = window.cursor_position().else_return()?;
        let cursor_position = camera
            .viewport_to_world_2d(camera_transform, cursor_position)
            .else_error()?;

        ghost_transform.translation.x = cursor_position.x;
        ghost_transform.translation.y = cursor_position.y;
    }

    fn update_haunted_sprite(
        ghost: Res<Ghost>,
        asset_server: Res<AssetServer>,
        debug_spawn: Res<DebugSpawn>,
        mut sprites: Query<(&mut Sprite, &mut Object)>,
    ) {
        let ghost = ghost.0.else_return()?;
        let (mut sprite, mut object) = sprites.get_mut(ghost).else_return()?;

        if object.id == debug_spawn.0 {
            return;
        }

        sprite.image = asset_server.load(OBJECTS[debug_spawn.0 as usize][0]);
        object.id = debug_spawn.0;
    }
}

fn debug_print_transform(on: On<PerformAction>, transforms: Query<&Transform>) {
    if !matches!(on.action, Action::DebugPrintTransform) {
        return;
    }

    let transform = transforms.get(on.target).else_error()?;
    info!("debug_print_transform: {}", transform.translation);
}

#[derive(Component)]
#[component(on_add = Self::on_add)]
#[require(Pickable {
    should_block_lower: true,
    is_hoverable: true,
})]
pub struct TargetFor(pub Vec<Action>);
impl TargetFor {
    pub fn new(targets: impl IntoIterator<Item = Action>) -> Self {
        Self(
            targets
                .into_iter()
                .chain([
                    Action::DebugSpawn,
                    Action::DebugPrintTransform,
                    Action::ChangeState,
                    Action::DeleteObject,
                ])
                .collect(),
        )
    }

    pub fn on_add(mut world: DeferredWorld, context: HookContext) {
        world
            .commands()
            .entity(context.entity)
            .observe(
                |on: On<PointerOver>,
                 target_fors: Query<&TargetFor>,
                 performing_action: Res<PerformingAction>,
                 mut cursor_icon: Single<&mut CursorIcon>| {
                    let performing_action = performing_action.0.as_ref().else_return()?;
                    let target_for = target_fors.get(on.entity).else_error()?;

                    if !target_for.0.contains(performing_action) {
                        return;
                    }

                    **cursor_icon = CursorIcon::System(SystemCursorIcon::Default);
                },
            )
            .observe(
                |on: On<PointerOut>,
                 target_fors: Query<&TargetFor>,
                 performing_action: Res<PerformingAction>,
                 mut cursor_icon: Single<&mut CursorIcon>| {
                    let performing_action = performing_action.0.as_ref().else_return()?;
                    let target_for = target_fors.get(on.entity).else_error()?;

                    if !target_for.0.contains(performing_action) {
                        return;
                    }

                    **cursor_icon = CursorIcon::System(SystemCursorIcon::NotAllowed);
                },
            )
            .observe(
                |on: On<PointerClick>,
                 target_fors: Query<&TargetFor>,
                 mut performing_action: ResMut<PerformingAction>,
                 mut cursor_icon: Single<&mut CursorIcon>,
                 mut commands: Commands,
                 background_colours: Query<
                    (Entity, &mut BackgroundColor, &Children),
                    With<SelectionAction>,
                >,
                 mut text_colours: Query<&mut TextColor>,
                 selected: Res<Selected>| {
                    let performing_action_unwrapped = performing_action.0.as_ref().else_return()?;
                    let target_for = target_fors.get(on.entity).else_error()?;

                    if !target_for.0.contains(performing_action_unwrapped) {
                        return;
                    }

                    **cursor_icon = CursorIcon::System(SystemCursorIcon::Default);
                    commands.trigger(PerformAction {
                        action: performing_action.0.take().else_error()?,
                        performer: selected.0,
                        target: on.entity,
                        position: on.hit.position.else_error()?.xy(),
                    });

                    for (entity, mut background_colour, children) in background_colours {
                        if entity == on.entity {
                            continue;
                        }

                        let mut text_colour = text_colours
                            .get_mut(*children.first().else_error()?)
                            .else_error()?;

                        background_colour.0 = WHITE.into();
                        text_colour.0 = BLACK.into();
                    }
                },
            );
    }
}

#[derive(Event)]
pub struct PerformAction {
    pub action: Action,
    pub performer: Entity,
    pub target: Entity,
    pub position: Vec2,
}
