use bevy::{input_management::Input, prelude::*};
use bevy_common_assets::ron::RonAssetPlugin;
use serde::{Deserialize, Serialize};

use crate::{
    input::SaveObjects,
    objects::{Object, SpawnObject},
};

pub fn plugin(app: &mut App) {
    app.add_plugins(RonAssetPlugin::<Save>::new(&[""]))
        .init_resource::<Loading>()
        .add_systems(Update, (save_objects, loading));
}

#[derive(Deserialize, Serialize, Asset, TypePath)]
struct Save {
    objects: Vec<SavedObject>,
}

#[derive(Deserialize, Serialize)]
struct SavedObject {
    id: u8,
    state: u8,
    position: Vec2,
}

fn save_objects(objects: Query<(&Object, &Transform)>, input: Res<Input>) {
    if !input.pressed::<SaveObjects>() {
        return;
    }

    let objects = objects
        .iter()
        .map(|(object, transform)| SavedObject {
            id: object.id,
            state: object.state,
            position: transform.translation.xy(),
        })
        .collect();

    let save = Save { objects };
    let save = ron::to_string(&save).else_error()?;

    std::fs::write("assets/save", save).else_error()?;
    info!("Saved!");
}

#[derive(Default, Resource)]
enum Loading {
    #[default]
    NotStarted,
    Waiting(Handle<Save>),
    Finished,
}

fn loading(
    mut loading: ResMut<Loading>,
    asset_server: Res<AssetServer>,
    save: Res<Assets<Save>>,
    mut commands: Commands,
) {
    match &*loading {
        Loading::NotStarted => {
            *loading = Loading::Waiting(asset_server.load("save"));
        }
        Loading::Waiting(handle) => {
            let save = save.get(handle).else_return()?;

            for saved_object in save.objects.iter() {
                let entity = commands.spawn_empty().id();
                commands.trigger(SpawnObject {
                    entity,
                    id: saved_object.id,
                    state: saved_object.state,
                    position: saved_object.position,
                    ghost: false,
                });
            }

            *loading = Loading::Finished;
            info!("Loaded");
        }
        Loading::Finished => (),
    }
}
