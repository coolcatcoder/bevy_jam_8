use bevy::{color::palettes::css::LIME, prelude::*};

pub fn plugin(app: &mut App) {
    app.init_resource::<CurrentWires>()
        .add_systems(Update, temporary_gizmos);
}

#[derive(Resource, Default)]
pub struct CurrentWires {
    pub ghost: Option<Entity>,
    pub wire_manager: Option<Entity>,
}

#[derive(Component)]
pub struct WireManager(pub Vec<Entity>);

fn temporary_gizmos(
    wire_managers: Query<&WireManager>,
    mut gizmos: Gizmos,
    transforms: Query<&Transform>,
) {
    for wire_manager in wire_managers {
        let mut wires = wire_manager.0.iter().copied();
        let starting_entity = wires.next().else_error()?;
        let transform = transforms.get(starting_entity).else_error()?;

        let mut previous_position = transform.translation.xy();
        for wire in wires {
            let transform = transforms.get(wire).else_error()?;
            gizmos.line_2d(previous_position, transform.translation.xy(), LIME);
            previous_position = transform.translation.xy();
        }
    }
}
