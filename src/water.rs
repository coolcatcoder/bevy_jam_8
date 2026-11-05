use avian2d::prelude::*;
use bevy::{ecs::entity::EntityHashSet, prelude::*};

use crate::selection::{Action, TargetFor};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, enable_and_disable_collision);
}

#[derive(Component)]
pub struct Water(pub EntityHashSet, pub Entity);

fn enable_and_disable_collision(
    waters: Query<&Water>,
    sprites: Query<&Sprite>,
    mut commands: Commands,
) {
    for water in waters {
        for entity in water.0.iter().copied() {
            let sprite = sprites.get(entity).else_error()?;
            let colour = sprite.color.to_srgba();

            if colour.red <= 1. {
                commands.entity(water.1).remove::<ColliderDisabled>();
            } else {
                commands.entity(water.1).insert(ColliderDisabled);
            }
        }
    }
}

pub fn on_collision_start(
    on: On<CollisionStart>,
    mut waters: Query<&mut Water>,
    target_fors: Query<&TargetFor>,
) {
    let body2 = on.body2.else_error()?;
    let target_for = target_fors.get(body2).else_return()?;
    if !target_for.0.contains(&Action::Transmute) {
        return;
    }

    let mut water = waters.get_mut(on.body1.else_error()?).else_error()?;
    water.0.insert(body2);
    info!("Start");
}

pub fn on_collision_end(
    on: On<CollisionEnd>,
    mut waters: Query<&mut Water>,
    target_fors: Query<&TargetFor>,
) {
    let body2 = on.body2.else_error()?;
    let target_for = target_fors.get(body2).else_return()?;
    if !target_for.0.contains(&Action::Transmute) {
        return;
    }

    let mut water = waters.get_mut(on.body1.else_error()?).else_error()?;
    water.0.remove(&body2);
}
