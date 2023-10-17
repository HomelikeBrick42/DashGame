use crate::{
    window::{InitWindowInternals, WindowSize},
    Quad, {Camera, GlobalTransform},
};
use bevy::{
    ecs::schedule::ScheduleLabel,
    prelude::{
        resource_changed, App, DetectChanges, IntoSystemConfigs, Plugin, Query, Ref, Res, ResMut,
        Resource,
    },
};
use encase::{ShaderSize, ShaderType, UniformBuffer};

#[derive(ScheduleLabel, Debug, PartialEq, Eq, Clone, Hash)]
pub struct RenderSchedule;

#[derive(ShaderType)]
struct GpuCamera {
    x: f32,
    y: f32,
    aspect: f32,
    vertical_height: f32,
}

#[derive(ShaderType)]
struct GpuQuad {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(ShaderType)]
struct GpuQuads<'a> {
    #[size(runtime)]
    quads: &'a [GpuQuad],
}

#[derive(Resource)]
struct Renderer {
    camera_uniform_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    queue: wgpu::Queue,
    device: wgpu::Device,
    _adapter: wgpu::Adapter,
    surface_configuration: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface,
    _instance: wgpu::Instance,
}

pub struct RendererPlugin;

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

        let camera_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Uniform Buffer"),
            size: GpuCamera::SHADER_SIZE.get(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(GpuCamera::SHADER_SIZE),
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_uniform_buffer.as_entire_binding(),
            }],
        });

        app.insert_resource(Renderer {
            camera_uniform_buffer,
            camera_bind_group,
            queue,
            device,
            _adapter: adapter,
            surface_configuration,
            surface,
            _instance: instance,
        })
        .init_schedule(RenderSchedule)
        .add_systems(
            RenderSchedule,
            (
                (
                    on_resize.run_if(resource_changed::<WindowSize>()),
                    update_camera,
                    update_quads,
                ),
                render,
            )
                .chain(),
        );
    }
}

fn on_resize(mut renderer: ResMut<Renderer>, size: Res<WindowSize>) {
    renderer.surface_configuration.width = size.width().get().try_into().unwrap();
    renderer.surface_configuration.height = size.height().get().try_into().unwrap();
    renderer
        .surface
        .configure(&renderer.device, &renderer.surface_configuration);
}

fn update_camera(renderer: Res<Renderer>, camera: Query<(Ref<GlobalTransform>, Ref<Camera>)>) {
    let (global_transform, camera) = camera.get_single().unwrap();
    if !global_transform.is_changed() && !camera.is_changed() {
        return;
    }

    let transform = global_transform.transform();
    let gpu_camera = GpuCamera {
        x: transform.x,
        y: transform.y,
        aspect: renderer.surface_configuration.width as f32
            / renderer.surface_configuration.height as f32,
        vertical_height: camera.vertical_height,
    };

    let mut buffer = UniformBuffer::new([0u8; GpuCamera::SHADER_SIZE.get() as _]);
    buffer.write(&gpu_camera).unwrap();
    let buffer = buffer.into_inner();

    renderer
        .queue
        .write_buffer(&renderer.camera_uniform_buffer, 0, &buffer);
}

// TODO: find a way to only upload new quads
fn update_quads(mut _renderer: ResMut<Renderer>, quads: Query<(Ref<GlobalTransform>, Ref<Quad>)>) {
    let _gpu_quads = quads
        .into_iter()
        .map(|(global_transform, quad)| {
            let transform = global_transform.transform();
            GpuQuad {
                x: transform.x,
                y: transform.y,
                width: quad.width,
                height: quad.height,
            }
        })
        .collect::<Vec<_>>();
}

fn render(renderer: ResMut<Renderer>) {
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
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

        // TODO: pipelines
        render_pass.set_bind_group(0, &renderer.camera_bind_group, &[]);
    }
    renderer.queue.submit([encoder.finish()]);

    output.present();
}
