use crate::renderer::RenderSchedule;
use bevy::prelude::*;
use std::num::NonZeroUsize;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

#[derive(Resource)]
pub struct WindowSize {
    width: NonZeroUsize,
    height: NonZeroUsize,
}

impl WindowSize {
    pub fn width(&self) -> NonZeroUsize {
        self.width
    }

    pub fn height(&self) -> NonZeroUsize {
        self.height
    }
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
                        _ => {}
                    }
                }
                Event::MainEventsCleared => {
                    app.update();
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
            .set_runner(Self::runner);
    }
}
