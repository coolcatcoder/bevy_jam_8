use bevy::{input_management::Input, prelude::*};
use bevy_common_assets::ron::RonAssetPlugin;
use foldhash::HashMap;

use crate::{
    input::SaveObjects,
    moss::WireManager,
    objects::{Object, SpawnObject},
    puzzles::PuzzleElements,
    save_format::{MetaSave, SAVE_VERSION, Save, SavedObject},
};

pub fn plugin(app: &mut App) {
    app.add_plugins(RonAssetPlugin::<MetaSave>::new(&[""]))
        .init_resource::<Loading>()
        .add_systems(Update, (save_objects, loading));
}

fn save_objects(
    objects: Query<(Entity, &Object, &Transform)>,
    input: Res<Input>,
    puzzle_elements: Res<PuzzleElements>,
    wire_managers: Query<&WireManager>,
) {
    if !input.pressed::<SaveObjects>() {
        return;
    }

    let mut next_index: u32 = 0;
    let mut entity_to_index: HashMap<Entity, u32> = HashMap::default();

    let objects = objects
        .iter()
        .filter_map(|(entity, object, transform)| {
            if object.ghost {
                return None;
            }

            entity_to_index.insert(entity, next_index);
            next_index += 1;

            Some(SavedObject {
                id: object.id,
                state: object.state,
                position: transform.translation.xy(),
            })
        })
        .collect();

    let puzzle_elements = puzzle_elements
        .entity_to_element
        .iter()
        .map(|(entity, puzzle_element)| (*entity_to_index.get(entity).unwrap(), *puzzle_element))
        .collect();

    let wires = wire_managers
        .iter()
        .map(|wire_manager| {
            wire_manager
                .0
                .iter()
                .map(|entity| *entity_to_index.get(entity).unwrap())
                .collect()
        })
        .collect();

    let save = Save {
        objects,
        puzzle_elements,
        wires,
    };
    let save = MetaSave {
        version: SAVE_VERSION,
        save: ron::to_string(&save).else_error()?,
    };
    let save = ron::to_string(&save).else_error()?;

    std::fs::write("assets/save", save).else_error()?;
    info!("Saved!");
}

#[derive(Default, Resource)]
enum Loading {
    #[default]
    NotStarted,
    Waiting(Handle<MetaSave>),
    Finished,
}

fn loading(
    mut loading: ResMut<Loading>,
    asset_server: Res<AssetServer>,
    save: Res<Assets<MetaSave>>,
    mut commands: Commands,
    mut puzzle_elements: ResMut<PuzzleElements>,
) {
    match &*loading {
        Loading::NotStarted => {
            *loading = Loading::Waiting(asset_server.load("save"));
        }
        Loading::Waiting(handle) => {
            let save = save.get(handle).else_return()?;
            let save = save.to_latest();

            let mut index_to_entity = vec![];

            for saved_object in save.objects.iter() {
                let entity = commands.spawn_empty().id();
                commands.trigger(SpawnObject {
                    entity,
                    id: saved_object.id,
                    state: saved_object.state,
                    position: saved_object.position,
                    ghost: false,
                });
                index_to_entity.push(entity);
            }

            for (entity, puzzle_element) in save.puzzle_elements.iter() {
                let entity = index_to_entity[*entity as usize];
                puzzle_elements
                    .entity_to_element
                    .insert(entity, *puzzle_element);
                puzzle_elements
                    .element_to_entity
                    .insert(*puzzle_element, entity);
            }

            for wires in save.wires.iter() {
                let wires = wires
                    .iter()
                    .map(|index| index_to_entity[*index as usize])
                    .collect();

                commands.spawn(WireManager(wires));
            }

            *loading = Loading::Finished;
            info!("Loaded");
        }
        Loading::Finished => (),
    }
}
