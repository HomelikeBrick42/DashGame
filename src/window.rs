use crate::renderer::RenderSchedule;
use bevy::prelude::*;
use enum_map::{Enum, EnumMap};
use std::{cmp::Ordering, num::NonZeroUsize};
use winit::{
    event::{ElementState, Event, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowSize {
    width: NonZeroUsize,
    height: NonZeroUsize,
}

impl WindowSize {
    #[inline]
    pub fn width(&self) -> NonZeroUsize {
        self.width
    }

    #[inline]
    pub fn height(&self) -> NonZeroUsize {
        self.height
    }
}

#[derive(Enum, Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct MouseButtons {
    buttons: EnumMap<MouseButton, bool>,
    pressed_buttons: EnumMap<MouseButton, bool>,
    released_buttons: EnumMap<MouseButton, bool>,
}

impl MouseButtons {
    pub fn is_button_down(&self, button: MouseButton) -> bool {
        self.buttons[button]
    }

    pub fn was_button_pressed(&self, button: MouseButton) -> bool {
        self.pressed_buttons[button]
    }

    pub fn was_button_released(&self, button: MouseButton) -> bool {
        self.released_buttons[button]
    }
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct MousePosition {
    x: f64,
    y: f64,
}

impl MousePosition {
    #[inline]
    pub fn x(&self) -> f64 {
        self.x
    }

    #[inline]
    pub fn y(&self) -> f64 {
        self.y
    }
}

#[derive(Event, Debug, Clone, Copy)]
pub struct MouseMovement {
    pub x: f64,
    pub y: f64,
    pub delta_x: f64,
    pub delta_y: f64,
}

#[derive(Event, Debug, Clone, Copy)]
pub enum MouseScroll {
    Up,
    Down,
}

pub(crate) struct InitWindowInternals {
    pub(crate) window: Window,
    event_loop: EventLoop<()>,
}

pub struct WindowPlugin;

impl WindowPlugin {
    fn runner(mut app: App) {
        let InitWindowInternals { window, event_loop } =
            app.world.remove_non_send_resource().unwrap();
        window.set_visible(true);
        let mut last_mouse_position: Option<winit::dpi::PhysicalPosition<f64>> = None;
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            match event {
                Event::WindowEvent { window_id, event } if window_id == window.id() => {
                    match event {
                        WindowEvent::Resized(_) => {
                            let size = window.inner_size();
                            *app.world.resource_mut::<WindowSize>() = WindowSize {
                                width: NonZeroUsize::new(size.width as usize)
                                    .unwrap_or(NonZeroUsize::MIN),
                                height: NonZeroUsize::new(size.height as usize)
                                    .unwrap_or(NonZeroUsize::MIN),
                            };
                        }
                        WindowEvent::Destroyed | WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                        }
                        WindowEvent::MouseWheel {
                            device_id: _,
                            delta,
                            phase: _,
                            ..
                        } => {
                            let y = match delta {
                                MouseScrollDelta::LineDelta(_, y) => y,
                                MouseScrollDelta::PixelDelta(delta) => delta.y as f32,
                            };
                            match y.partial_cmp(&0.0) {
                                Some(Ordering::Less) => app.world.send_event(MouseScroll::Down),
                                Some(Ordering::Greater) => app.world.send_event(MouseScroll::Up),
                                None | Some(Ordering::Equal) => {}
                            }
                        }
                        WindowEvent::CursorLeft { device_id: _ } => {
                            last_mouse_position = None;
                        }
                        WindowEvent::CursorMoved {
                            device_id: _,
                            position: mouse_position,
                            ..
                        } => {
                            let (delta_x, delta_y) =
                                last_mouse_position.map_or((0.0, 0.0), |last_mouse_position| {
                                    (
                                        mouse_position.x - last_mouse_position.x,
                                        mouse_position.y - last_mouse_position.y,
                                    )
                                });
                            app.world.send_event(MouseMovement {
                                x: mouse_position.x,
                                y: mouse_position.y,
                                delta_x,
                                delta_y,
                            });
                            *app.world.resource_mut::<MousePosition>() = MousePosition {
                                x: mouse_position.x,
                                y: mouse_position.y,
                            };
                            last_mouse_position = Some(mouse_position);
                        }
                        WindowEvent::MouseInput {
                            device_id: _,
                            state,
                            button,
                            ..
                        } => {
                            let mut mouse_buttons = app.world.resource_mut::<MouseButtons>();
                            match button {
                                winit::event::MouseButton::Left => {
                                    mouse_buttons.buttons[MouseButton::Left] =
                                        state == ElementState::Pressed;
                                    mouse_buttons.pressed_buttons[MouseButton::Left] =
                                        state == ElementState::Pressed;
                                }
                                winit::event::MouseButton::Right => {
                                    mouse_buttons.buttons[MouseButton::Right] =
                                        state == ElementState::Pressed;
                                    mouse_buttons.pressed_buttons[MouseButton::Right] =
                                        state == ElementState::Pressed;
                                }
                                winit::event::MouseButton::Middle => {
                                    mouse_buttons.buttons[MouseButton::Middle] =
                                        state == ElementState::Pressed;
                                    mouse_buttons.pressed_buttons[MouseButton::Middle] =
                                        state == ElementState::Pressed;
                                }
                                winit::event::MouseButton::Other(_) => {}
                            }
                        }
                        _ => {}
                    }
                }
                Event::MainEventsCleared => {
                    app.update();
                    let mut mouse_buttons = app.world.resource_mut::<MouseButtons>();
                    mouse_buttons.pressed_buttons = EnumMap::default();
                    mouse_buttons.released_buttons = EnumMap::default();
                    window.request_redraw();
                }
                Event::RedrawRequested(window_id) if window_id == window.id() => {
                    app.world.run_schedule(RenderSchedule);
                }
                _ => {}
            }
        });
    }
}

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Dash Game")
            .with_visible(false)
            .build(&event_loop)
            .unwrap();

        app.insert_non_send_resource(InitWindowInternals { event_loop, window })
            .insert_resource(WindowSize {
                width: NonZeroUsize::MIN,
                height: NonZeroUsize::MIN,
            })
            .insert_resource(MousePosition { x: 0.0, y: 0.0 })
            .insert_resource(MouseButtons {
                buttons: EnumMap::default(),
                pressed_buttons: EnumMap::default(),
                released_buttons: EnumMap::default(),
            })
            .add_event::<MouseMovement>()
            .add_event::<MouseScroll>()
            .set_runner(Self::runner);
    }
}
