#![allow(unused)]

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
    }
}
