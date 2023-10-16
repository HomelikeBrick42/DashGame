#![allow(clippy::type_complexity)]

pub mod renderer;
pub mod window;

use bevy::{app::PluginGroupBuilder, prelude::*};
use renderer::RendererPlugin;
use window::WindowPlugin;

pub struct GamePlugins;

impl PluginGroup for GamePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(WindowPlugin)
            .add(RendererPlugin)
            .add(TransformPlugin)
    }
}

// TODO: switch to motors
#[derive(Component, Clone, Copy)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Clone, Copy)]
pub struct GlobalTransform(pub(crate) Transform);

impl GlobalTransform {
    pub fn transform(&self) -> &Transform {
        &self.0
    }
}

struct TransformPlugin;

impl Plugin for TransformPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (add_global_transforms, update_global_transforms).chain(),
        );
    }
}

fn add_global_transforms(
    mut commands: Commands,
    transforms: Query<(Entity, &Transform), Without<GlobalTransform>>,
) {
    for (entity, &transform) in &transforms {
        commands
            .get_entity(entity)
            .unwrap()
            .insert(GlobalTransform(transform));
    }
}

fn update_global_transforms(
    mut transforms: Query<(&mut GlobalTransform, &Transform), Changed<Transform>>,
) {
    for (mut global_transform, &transform) in &mut transforms {
        global_transform.0 = transform;
    }
}

#[derive(Component, Clone, Copy)]
pub struct Quad {
    pub width: f32,
    pub height: f32,
}

#[derive(Component, Clone, Copy)]
pub struct Camera {
    pub vertical_height: f32,
}
