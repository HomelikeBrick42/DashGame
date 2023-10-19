use crate::{
    window::{InitWindowInternals, WindowSize},
    Circle, Material, Quad, {Camera, GlobalTransform},
};
use bevy::{
    ecs::schedule::ScheduleLabel,
    prelude::{
        resource_changed, App, DetectChanges, IntoSystemConfigs, Plugin, Query, Ref, Res, ResMut,
        Resource,
    },
};
use encase::{DynamicStorageBuffer, ShaderSize, ShaderType, UniformBuffer};
use wgpu::include_wgsl;

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
    red: f32,
    green: f32,
    blue: f32,
}

#[derive(ShaderType)]
struct GpuCircle {
    x: f32,
    y: f32,
    radius: f32,
    red: f32,
    green: f32,
    blue: f32,
}

#[derive(Resource)]
struct Renderer {
    circle_render_pipeline: wgpu::RenderPipeline,
    circle_buffer: wgpu::Buffer,
    circle_buffer_size: wgpu::BufferAddress,
    circle_count: u32,
    circle_bind_group_layout: wgpu::BindGroupLayout,
    circle_bind_group: wgpu::BindGroup,
    quad_render_pipeline: wgpu::RenderPipeline,
    quad_buffer: wgpu::Buffer,
    quad_buffer_size: wgpu::BufferAddress,
    quad_count: u32,
    quad_bind_group_layout: wgpu::BindGroupLayout,
    quad_bind_group: wgpu::BindGroup,
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

        let quad_buffer_size = GpuQuad::SHADER_SIZE;
        let quad_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Quad Storage Buffer"),
            size: quad_buffer_size.get(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let quad_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Quad Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(quad_buffer_size),
                    },
                    count: None,
                }],
            });
        let quad_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Quad Bind Group"),
            layout: &quad_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: quad_buffer.as_entire_binding(),
            }],
        });

        let quad_shader = device.create_shader_module(include_wgsl!("./quad_shader.wgsl"));

        let quad_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Quad Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &quad_bind_group_layout],
                push_constant_ranges: &[],
            });
        let quad_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Quad Render Pipeline"),
            layout: Some(&quad_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &quad_shader,
                entry_point: "vertex",
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &quad_shader,
                entry_point: "pixel",
                targets: &[Some(surface_configuration.format.into())],
            }),
            multiview: None,
        });

        let circle_buffer_size = GpuCircle::SHADER_SIZE;
        let circle_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Circle Storage Buffer"),
            size: circle_buffer_size.get(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let circle_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Circle Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(circle_buffer_size),
                    },
                    count: None,
                }],
            });
        let circle_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Circle Bind Group"),
            layout: &circle_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: circle_buffer.as_entire_binding(),
            }],
        });

        let circle_shader = device.create_shader_module(include_wgsl!("./circle_shader.wgsl"));

        let circle_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Circle Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &circle_bind_group_layout],
                push_constant_ranges: &[],
            });
        let circle_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Circle Render Pipeline"),
                layout: Some(&circle_render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &circle_shader,
                    entry_point: "vertex",
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &circle_shader,
                    entry_point: "pixel",
                    targets: &[Some(surface_configuration.format.into())],
                }),
                multiview: None,
            });

        app.insert_resource(Renderer {
            circle_render_pipeline,
            circle_buffer,
            circle_buffer_size: circle_buffer_size.get(),
            circle_count: 0,
            circle_bind_group,
            circle_bind_group_layout,
            quad_render_pipeline,
            quad_buffer,
            quad_buffer_size: quad_buffer_size.get(),
            quad_count: 0,
            quad_bind_group,
            quad_bind_group_layout,
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
                    update_circles,
                ),
                render,
            )
                .chain(),
        );
    }
}

fn on_resize(mut renderer: ResMut<'_, Renderer>, size: Res<'_, WindowSize>) {
    renderer.surface_configuration.width = size.width().get().try_into().unwrap();
    renderer.surface_configuration.height = size.height().get().try_into().unwrap();
    renderer
        .surface
        .configure(&renderer.device, &renderer.surface_configuration);
}

fn update_camera(
    renderer: Res<'_, Renderer>,
    camera: Query<'_, '_, (Ref<'_, GlobalTransform>, Ref<'_, Camera>)>,
    size: Res<'_, WindowSize>,
) {
    let (global_transform, camera) = camera.get_single().unwrap();
    if !global_transform.is_changed() && !camera.is_changed() && !size.is_changed() {
        return;
    }

    let transform = global_transform.transform();
    let gpu_camera = GpuCamera {
        x: transform.x,
        y: transform.y,
        aspect: size.width().get() as f32 / size.height().get() as f32,
        vertical_height: camera.vertical_height,
    };

    let mut buffer = UniformBuffer::new([0u8; GpuCamera::SHADER_SIZE.get() as _]);
    buffer.write(&gpu_camera).unwrap();
    let buffer = buffer.into_inner();

    renderer
        .queue
        .write_buffer(&renderer.camera_uniform_buffer, 0, &buffer);
}

// TODO: find a way to only upload changed quads
fn update_quads(
    mut renderer: ResMut<'_, Renderer>,
    quads: Query<
        '_,
        '_,
        (
            Ref<'_, GlobalTransform>,
            Ref<'_, Quad>,
            Option<Ref<'_, Material>>,
        ),
    >,
) {
    let mut anything_changed = false;
    let mut quad_count = 0usize;
    let mut buffer = DynamicStorageBuffer::new(vec![]);
    for (global_transform, quad, material) in &quads {
        quad_count += 1;
        anything_changed |= global_transform.is_changed() || quad.is_changed();
        let transform = global_transform.transform();
        let (red, green, blue) = material.map_or((1.0, 1.0, 1.0), |material| {
            anything_changed |= material.is_changed();
            let Material { red, green, blue } = *material;
            (red, green, blue)
        });
        buffer
            .write(&GpuQuad {
                x: transform.x,
                y: transform.y,
                width: quad.width,
                height: quad.height,
                red,
                green,
                blue,
            })
            .unwrap();
    }

    let quad_count = quad_count.try_into().unwrap();
    if anything_changed || quad_count != renderer.quad_count {
        renderer.quad_count = quad_count;
        let buffer = buffer.into_inner();

        let required_buffer_size: wgpu::BufferAddress = buffer.len().try_into().unwrap();
        if required_buffer_size > renderer.quad_buffer_size {
            renderer.quad_buffer_size = required_buffer_size;
            renderer.quad_buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Quad Storage Buffer"),
                size: renderer.quad_buffer_size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            });
            renderer.quad_bind_group =
                renderer
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Quad Bind Group"),
                        layout: &renderer.quad_bind_group_layout,
                        entries: &[wgpu::BindGroupEntry {
                            binding: 0,
                            resource: renderer.quad_buffer.as_entire_binding(),
                        }],
                    });
        }

        renderer
            .queue
            .write_buffer(&renderer.quad_buffer, 0, &buffer);
    }
}

// TODO: find a way to only upload changed circles
fn update_circles(
    mut renderer: ResMut<'_, Renderer>,
    circles: Query<
        '_,
        '_,
        (
            Ref<'_, GlobalTransform>,
            Ref<'_, Circle>,
            Option<Ref<'_, Material>>,
        ),
    >,
) {
    let mut anything_changed = false;
    let mut circle_count = 0usize;
    let mut buffer = DynamicStorageBuffer::new(vec![]);
    for (global_transform, circle, material) in &circles {
        circle_count += 1;
        anything_changed |= global_transform.is_changed() || circle.is_changed();
        let transform = global_transform.transform();
        let (red, green, blue) = material.map_or((1.0, 1.0, 1.0), |material| {
            anything_changed |= material.is_changed();
            let Material { red, green, blue } = *material;
            (red, green, blue)
        });
        buffer
            .write(&GpuCircle {
                x: transform.x,
                y: transform.y,
                radius: circle.radius,
                red,
                green,
                blue,
            })
            .unwrap();
    }

    let circle_count = circle_count.try_into().unwrap();
    if anything_changed || circle_count != renderer.circle_count {
        renderer.circle_count = circle_count;
        let buffer = buffer.into_inner();

        let required_buffer_size: wgpu::BufferAddress = buffer.len().try_into().unwrap();
        if required_buffer_size > renderer.circle_buffer_size {
            renderer.circle_buffer_size = required_buffer_size;
            renderer.circle_buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Circle Storage Buffer"),
                size: renderer.circle_buffer_size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            });
            renderer.circle_bind_group =
                renderer
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Circle Bind Group"),
                        layout: &renderer.circle_bind_group_layout,
                        entries: &[wgpu::BindGroupEntry {
                            binding: 0,
                            resource: renderer.circle_buffer.as_entire_binding(),
                        }],
                    });
        }

        renderer
            .queue
            .write_buffer(&renderer.circle_buffer, 0, &buffer);
    }
}

fn render(renderer: ResMut<'_, Renderer>) {
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

        render_pass.set_pipeline(&renderer.quad_render_pipeline);
        render_pass.set_bind_group(0, &renderer.camera_bind_group, &[]);
        render_pass.set_bind_group(1, &renderer.quad_bind_group, &[]);
        render_pass.draw(0..4, 0..renderer.quad_count);

        render_pass.set_pipeline(&renderer.circle_render_pipeline);
        render_pass.set_bind_group(1, &renderer.circle_bind_group, &[]);
        render_pass.draw(0..4, 0..renderer.circle_count);
    }
    renderer.queue.submit([encoder.finish()]);

    output.present();
}
