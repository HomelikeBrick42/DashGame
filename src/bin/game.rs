use bevy::prelude::*;
use dash_game::GamePlugins;

fn main() {
    App::new().add_plugins(GamePlugins).run();
}
