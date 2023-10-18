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

impl Transform {
    #[inline]
    pub fn apply(self, other: Transform) -> Transform {
        Transform {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct GlobalTransform(pub(crate) Transform);

impl GlobalTransform {
    #[inline]
    pub fn transform(&self) -> &Transform {
        &self.0
    }
}

struct TransformPlugin;

impl Plugin for TransformPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                remove_global_transforms,
                update_global_transforms,
                add_global_transforms,
            )
                .chain(),
        );
    }
}

fn add_global_transforms(
    mut commands: Commands,
    transforms: Query<(Entity, &Transform, Option<&Parent>), Without<GlobalTransform>>,
) {
    for (entity, &transform, mut maybe_parent) in &transforms {
        let mut final_transform = transform;
        while let Some(parent) = maybe_parent {
            let Ok((_, &parent_transform, parent)) = transforms.get(parent.get()) else {
                break;
            };
            final_transform = final_transform.apply(parent_transform);
            maybe_parent = parent;
        }
        commands
            .get_entity(entity)
            .unwrap()
            .insert(GlobalTransform(transform));
    }
}

fn remove_global_transforms(
    mut commands: Commands,
    global_transforms_without_transform: Query<Entity, (With<GlobalTransform>, Without<Transform>)>,
) {
    for entity in &global_transforms_without_transform {
        commands
            .get_entity(entity)
            .unwrap()
            .remove::<GlobalTransform>();
    }
}

// TODO: find some way to avoid updating transforms if nothing has changed
fn update_global_transforms(
    transforms: Query<(Ref<Transform>, Option<&Parent>)>,
    mut global_transforms: Query<(&mut GlobalTransform, Ref<Transform>, Option<&Parent>)>,
) {
    global_transforms.par_iter_mut().for_each_mut(
        |(mut global_transform, transform, mut maybe_parent)| {
            let mut any_transform_changed = transform.is_changed();
            let mut final_transform = *transform;
            while let Some(parent) = maybe_parent {
                let Ok((parent_transform, parent)) = transforms.get(parent.get()) else {
                    break;
                };
                any_transform_changed |= parent_transform.is_changed();
                final_transform = final_transform.apply(*parent_transform);
                maybe_parent = parent;
            }
            if any_transform_changed {
                global_transform.0 = final_transform;
            }
        },
    );
}

#[derive(Component, Clone, Copy)]
pub struct Quad {
    pub width: f32,
    pub height: f32,
}

#[derive(Component, Clone, Copy)]
pub struct Circle {
    pub radius: f32,
}

#[derive(Component, Clone, Copy)]
pub struct Material {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

#[derive(Component, Clone, Copy)]
pub struct Camera {
    pub vertical_height: f32,
}
