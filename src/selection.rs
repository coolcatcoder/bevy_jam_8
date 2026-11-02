use bevy::{
    color::palettes::css::{BLACK, LIME, WHITE},
    prelude::*,
    window::{CursorIcon, SystemCursorIcon},
};

use crate::{objects::SpawnObject, ui::SelectionAction};

pub fn plugin(app: &mut App) {
    app.add_observer(MoveTowards::perform_action)
        .init_resource::<PerformingAction>()
        .init_resource::<DebugSpawn>()
        .add_observer(DebugSpawn::perform_action)
        .add_systems(
            Update,
            (
                temporary_gizmo_that_should_be_replaced_with_a_sprite_of_a_pre_rendered_torus,
                MoveTowards::motion,
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
}
impl Action {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Move => "Move.",
            Self::DebugSpawn => "Debug spawn.",
        }
    }
}

#[derive(Component)]
struct MoveTowards(Vec2);
impl MoveTowards {
    fn perform_action(on: On<PerformAction>, mut commands: Commands) {
        if !matches!(on.action, Action::Move) {
            return;
        }

        commands
            .entity(on.performer)
            .insert(MoveTowards(on.position));
    }

    fn motion(
        move_towardss: Query<(Entity, &MoveTowards, &mut Transform)>,
        mut commands: Commands,
        time: Res<Time>,
    ) {
        let time = time.delta_secs();
        for (entity, move_towards, mut transform) in move_towardss {
            const SPEED: f32 = 70.;

            let direction = (move_towards.0 - transform.translation.xy()).normalize_or_zero();
            let velocity = direction * SPEED;

            let displacement = velocity * time;
            transform.translation.x += displacement.x;
            transform.translation.y += displacement.y;

            if transform.translation.xy().distance_squared(move_towards.0) < (5. * 5.) {
                commands.entity(entity).remove::<MoveTowards>();
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct DebugSpawn(pub u8);
impl DebugSpawn {
    fn perform_action(on: On<PerformAction>, mut commands: Commands, debug_spawn: Res<DebugSpawn>) {
        if !matches!(on.action, Action::DebugSpawn) {
            return;
        }

        let entity = commands.spawn_empty().id();
        commands.trigger(SpawnObject {
            entity,
            id: debug_spawn.0,
            state: 0,
            position: on.position,
        });
    }
}

#[derive(Component)]
#[component(on_add = Self::on_add)]
#[require(Pickable {
    should_block_lower: true,
    is_hoverable: true,
})]
pub struct TargetFor(pub Vec<Action>);
impl TargetFor {
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
