use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Asset, TypePath)]
pub struct MetaSave {
    pub version: Version,
    pub save: String,
}
impl MetaSave {
    pub fn to_latest(&self) -> Save {
        match self.version {
            Version::V1 => {
                let save: Save1 = ron::from_str(&self.save).unwrap();
                save.into_latest()
            }
            Version::V0 => {
                let save: Save0 = ron::from_str(&self.save).unwrap();
                save.into_latest()
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
pub enum Version {
    V1,
    V0,
}

pub const SAVE_VERSION: Version = Version::V1;
pub type Save = Save1;
pub type SavedObject = SavedObject1;

#[derive(Deserialize, Serialize)]
pub struct Save1 {
    pub objects: Vec<SavedObject1>,
    pub puzzle_elements: Vec<(u32, u8)>,
    pub wires: Vec<Vec<u32>>,
}
impl Save1 {
    fn into_latest(self) -> Save {
        self
    }
}
#[derive(Deserialize, Serialize)]
pub struct SavedObject1 {
    pub id: u8,
    pub state: u8,
    pub position: Vec2,
}

#[derive(Deserialize, Serialize)]
pub struct Save0 {
    pub objects: Vec<SavedObject0>,
    pub puzzle_elements: Vec<(u32, u8)>,
}
impl Save0 {
    fn into_latest(self) -> Save {
        Save1 {
            objects: self
                .objects
                .into_iter()
                .map(|object| SavedObject1 {
                    id: object.id,
                    state: object.state,
                    position: object.position,
                })
                .collect(),
            puzzle_elements: self.puzzle_elements,
            wires: vec![],
        }
    }
}
#[derive(Deserialize, Serialize)]
pub struct SavedObject0 {
    pub id: u8,
    pub state: u8,
    pub position: Vec2,
}
