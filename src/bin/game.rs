use bevy::prelude::{App, Commands, EventReader, Query, Res, Startup, Update, With};
use dash_game::{
    window::{MouseButton, MouseButtons, MouseMovement, WindowSize},
    Camera, Circle, GamePlugins, Material, Quad, Transform,
};

fn main() {
    App::new()
        .add_plugins(GamePlugins)
        .add_systems(Startup, startup)
        .add_systems(Update, mouse_movement)
        .run();
}

fn startup(mut commands: Commands) {
    let _camera = commands.spawn((
        Transform { x: 0.0, y: 0.0 },
        Camera {
            vertical_height: 2.0,
        },
    ));
    let _quad = commands.spawn((
        Transform { x: -0.6, y: 0.0 },
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
    let _circle = commands.spawn((
        Transform { x: 0.6, y: 0.0 },
        Circle { radius: 0.5 },
        Material {
            red: 0.2,
            green: 0.0,
            blue: 1.0,
        },
    ));
}

fn mouse_movement(
    mut camera: Query<&mut Transform, With<Camera>>,
    mut mouse_movement_events: EventReader<MouseMovement>,
    size: Res<WindowSize>,
    mouse_buttons: Res<MouseButtons>,
) {
    let mut camera_transform = camera.get_single_mut().unwrap();
    let aspect = size.width().get() as f32 / size.height().get() as f32;
    for mouse_movement in mouse_movement_events.iter() {
        if mouse_buttons.is_button_down(MouseButton::Right) {
            camera_transform.x +=
                -mouse_movement.delta_x as f32 / size.width().get() as f32 * aspect * 2.0;
            camera_transform.y += mouse_movement.delta_y as f32 / size.height().get() as f32 * 2.0;
        }
    }
}
