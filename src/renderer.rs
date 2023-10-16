use crate::window::{InitWindowInternals, WindowSize};
use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

#[derive(ScheduleLabel, Debug, PartialEq, Eq, Clone, Hash)]
pub struct RenderSchedule;

#[derive(Resource)]
struct Renderer {
    queue: wgpu::Queue,
    device: wgpu::Device,
    adapter: wgpu::Adapter,
    surface_configuration: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface,
    instance: wgpu::Instance,
}

pub struct RendererPlugin;

impl RendererPlugin {
    fn on_resize(mut renderer: ResMut<Renderer>, size: Res<WindowSize>) {
        renderer.surface_configuration.width = size.width().get().try_into().unwrap();
        renderer.surface_configuration.height = size.height().get().try_into().unwrap();
        renderer
            .surface
            .configure(&renderer.device, &renderer.surface_configuration);
    }

    fn render(mut renderer: ResMut<Renderer>) {
        let output = loop {
            match renderer.surface.get_current_texture() {
                Ok(output) => break output,
                Err(wgpu::SurfaceError::Timeout) => return, // give up on rendering for now
                Err(wgpu::SurfaceError::Outdated) => {
                    renderer
                        .surface
                        .configure(&renderer.device, &renderer.surface_configuration);
                }
                Err(wgpu::SurfaceError::Lost) => panic!("wgpu device lost"),
                Err(wgpu::SurfaceError::OutOfMemory) => panic!("wgpu is out of memory"),
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = renderer
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
        }
        renderer.queue.submit([encoder.finish()]);

        output.present();
    }
}

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
        });

        let surface = unsafe {
            instance.create_surface(&app.world.non_send_resource::<InitWindowInternals>().window)
        }
        .unwrap();

        let (adapter, device, queue) = pollster::block_on(async {
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    force_fallback_adapter: false,
                    compatible_surface: Some(&surface),
                })
                .await
                .unwrap();

            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: Some("Main Device"),
                        features: wgpu::Features::default(),
                        limits: wgpu::Limits::default(),
                    },
                    None,
                )
                .await
                .unwrap();

            (adapter, device, queue)
        });

        let window_size = app.world.get_resource::<WindowSize>().unwrap();
        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]);
        let surface_configuration = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width().get().try_into().unwrap(),
            height: window_size.height().get().try_into().unwrap(),
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &surface_configuration);

        app.insert_resource(Renderer {
            queue,
            device,
            adapter,
            surface_configuration,
            surface,
            instance,
        })
        .init_schedule(RenderSchedule)
        .add_systems(
            RenderSchedule,
            (
                Self::on_resize.run_if(resource_changed::<WindowSize>()),
                Self::render,
            )
                .chain(),
        );
    }
}
