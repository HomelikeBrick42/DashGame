#![deny(rust_2018_idioms)]

use bevy::prelude::{App, Commands, EventReader, Query, Res, Startup, Update};
use dash_game::{
    window::{MouseButton, MouseButtons, MouseMovement, MouseScroll, WindowSize},
    Camera, Circle, GamePlugins, Material, Quad, Transform,
};

fn main() {
    App::new()
        .add_plugins(GamePlugins)
        .add_systems(Startup, startup)
        .add_systems(Update, (camera_mouse_movement, camera_zoom))
        .run();
}

fn startup(mut commands: Commands<'_, '_>) {
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

fn camera_mouse_movement(
    mut camera: Query<'_, '_, (&mut Transform, &Camera)>,
    mut mouse_movement_events: EventReader<'_, '_, MouseMovement>,
    size: Res<'_, WindowSize>,
    mouse_buttons: Res<'_, MouseButtons>,
) {
    let (mut camera_transform, camera) = camera.get_single_mut().unwrap();
    let aspect = size.width().get() as f32 / size.height().get() as f32;
    for mouse_movement in mouse_movement_events.iter() {
        if mouse_buttons.is_button_down(MouseButton::Right) {
            camera_transform.x += -mouse_movement.delta_x as f32 / size.width().get() as f32
                * camera.vertical_height
                * aspect;
            camera_transform.y +=
                mouse_movement.delta_y as f32 / size.height().get() as f32 * camera.vertical_height;
        }
    }
}

fn camera_zoom(
    mut camera: Query<'_, '_, &mut Camera>,
    mut mouse_scroll_events: EventReader<'_, '_, MouseScroll>,
) {
    let mut camera = camera.get_single_mut().unwrap();
    for mouse_scroll_event in mouse_scroll_events.iter() {
        match mouse_scroll_event {
            MouseScroll::Up => camera.vertical_height *= 0.9,
            MouseScroll::Down => camera.vertical_height /= 0.9,
        }
    }
}
