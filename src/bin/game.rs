use bevy::prelude::{App, Commands, Startup};
use dash_game::{Camera, GamePlugins, Material, Quad, Transform};

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
    let _quad = commands.spawn((
        Transform { x: 0.0, y: 0.0 },
        Quad {
            width: 1.0,
            height: 1.0,
        },
        Material {
            red: 1.0,
            green: 0.2,
            blue: 0.0,
        },
    ));
}
