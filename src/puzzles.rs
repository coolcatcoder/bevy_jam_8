use bevy::{color::palettes::css::WHITE, ecs::system::SystemParam, prelude::*};
use foldhash::HashMap;

use crate::objects::{LoadedObjects, Object};

pub fn plugin(app: &mut App) {
    app.init_resource::<PuzzleElements>()
        .init_resource::<Won>()
        .add_observer(the_grand_puzzle_manager)
        .add_systems(Update, win_text);
}

#[derive(EntityEvent)]
pub struct StateChange(pub Entity);

#[derive(Resource, Default)]
pub struct PuzzleElements {
    pub entity_to_element: HashMap<Entity, u8>,
    pub element_to_entity: HashMap<u8, Entity>,
}

fn the_grand_puzzle_manager(
    on: On<StateChange>,
    manager: TheGrandPuzzleManager,
    mut commands: Commands,
    mut won: ResMut<Won>,
) {
    let puzzle_element = manager
        .puzzle_elements
        .entity_to_element
        .get(&on.0)
        .else_return()?;

    match puzzle_element {
        0 | 1 => {
            let statue_0 = manager.state(0);
            let statue_1 = manager.state(1);

            if statue_0 == 1 && statue_1 == 1 {
                for element in 2..=6 {
                    let entity = manager.element_to_entity(element);
                    commands.entity(entity).despawn();
                }
            }
        }
        7 | 8 => {
            let statue_7 = manager.state(7);
            let statue_8 = manager.state(8);

            if statue_7 == 1 && statue_8 == 1 {
                for element in 9..=12 {
                    let entity = manager.element_to_entity(element);
                    commands.entity(entity).despawn();
                }

                won.0 = true;
            }
        }
        _ => error!("Unknown puzzle element"),
    }
}

#[derive(SystemParam)]
struct TheGrandPuzzleManager<'w, 's> {
    puzzle_elements: Res<'w, PuzzleElements>,
    objects: Query<'w, 's, (&'static mut Object, &'static mut Sprite)>,
    //loaded_objects: Res<'w, LoadedObjects>,
}
impl TheGrandPuzzleManager<'_, '_> {
    fn element_to_entity(&self, element: u8) -> Entity {
        *self
            .puzzle_elements
            .element_to_entity
            .get(&element)
            .unwrap()
    }

    fn state(&self, element: u8) -> u8 {
        self.objects
            .get(
                *self
                    .puzzle_elements
                    .element_to_entity
                    .get(&element)
                    .unwrap(),
            )
            .unwrap()
            .0
            .state
    }

    fn set_state(&mut self, element: u8, state: u8) {
        self.objects
            .get_mut(
                *self
                    .puzzle_elements
                    .element_to_entity
                    .get(&element)
                    .unwrap(),
            )
            .unwrap()
            .0
            .state = state;
    }
}

#[derive(Resource, Default)]
struct Won(bool);

fn win_text(won: Res<Won>, mut gizmos: Gizmos) {
    if won.0 {
        gizmos.text_2d(
            Vec2(190., 400.),
            "That is all.\nThank you for playing.",
            5.,
            Vec2::ZERO,
            WHITE,
        );
    }
}
