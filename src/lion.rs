use avian2d::prelude::*;
use bevy::{ecs::entity::EntityHashMap, prelude::*};

use crate::selection::{Action, TargetFor};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, attract);
}

#[derive(Component)]
#[component(on_add = Self::on_add)]
pub struct Lion(pub bool, pub EntityHashMap<bool>);
impl Lion {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        world
            .commands()
            .entity(context.entity)
            .observe(collision_start)
            .observe(collision_end);
    }
}

fn attract(
    lions: Query<(&mut Lion, &Transform, &Children)>,
    spatial_query: SpatialQuery,
    mut metals: Query<(Ref<Transform>, &mut LinearVelocity, &LockedAxes)>,
    child_of: Query<&ChildOf>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let time = time.delta_secs();

    for (mut lion, lion_transform, children) in lions {
        if !lion.0 {
            continue;
        }

        for (entity, visible) in lion.1.iter_mut() {
            let (metal_transform, mut linear_velocity, locked_axes) =
                metals.get_mut(*entity).else_error()?;

            //if metal_transform.is_changed() {
            let direction = metal_transform.translation.xy() - lion_transform.translation.xy();
            let direction = Dir2::new(direction).else_error()?;

            let distance = metal_transform
                .translation
                .xy()
                .distance(lion_transform.translation.xy());

            let ray = spatial_query.cast_ray(
                lion_transform.translation.xy(),
                direction,
                distance,
                false,
                &SpatialQueryFilter::default().with_excluded_entities(children.iter()),
            );

            match ray {
                None => {
                    *visible = true;
                }
                Some(ray) => {
                    let ray_entity = child_of
                        .get(ray.entity)
                        .map(|child_of| child_of.0)
                        .unwrap_or(ray.entity);
                    *visible = ray_entity == *entity;
                }
            }
            //}

            if !*visible || locked_axes.is_translation_locked() {
                continue;
            }

            const DISTANCE: f32 = 1.;
            let direction = (lion_transform.translation.xy() - metal_transform.translation.xy())
                .normalize_or_zero();
            let displacement = direction * DISTANCE;

            linear_velocity.0 = displacement / time;
            commands.entity(*entity).remove::<Sleeping>();
        }
    }
}

fn collision_start(
    on: On<CollisionStart>,
    mut lions: Query<&mut Lion>,
    target_fors: Query<&TargetFor>,
) {
    let mut lion = lions.get_mut(on.body1.else_error()?).else_error()?;
    let body2 = on.body2.else_return()?;
    let target_for = target_fors.get(body2).else_return()?;

    if !target_for.0.contains(&Action::Transmute) {
        return;
    }

    lion.1.insert(body2, false);
}

fn collision_end(
    on: On<CollisionEnd>,
    mut lions: Query<&mut Lion>,
    target_fors: Query<&TargetFor>,
) {
    let mut lion = lions.get_mut(on.body1.else_error()?).else_error()?;
    let body2 = on.body2.else_return()?;
    let target_for = target_fors.get(body2).else_return()?;

    if !target_for.0.contains(&Action::Transmute) {
        return;
    }

    lion.1.remove(&body2);
}
