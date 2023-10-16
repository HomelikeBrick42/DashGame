use bevy::prelude::{App, Commands, Startup};
use dash_game::{Camera, GamePlugins, Transform};

fn main() {
    App::new()
        .add_plugins(GamePlugins)
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands) {
    let _camera = commands.spawn((
        Transform { x: 0.0, y: 0.0 },
        Camera {
            vertical_height: 1.0,
        },
    ));
}
