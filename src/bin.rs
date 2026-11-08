use bevy::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            bevy_jam_8::main_plugin,
        ))
        .run()
}
