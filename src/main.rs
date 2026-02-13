use bevy::prelude::*;

mod components;
mod events;
mod plugins;
mod resources;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "红中 Red Center".into(),
                        resolution: (1280, 720).into(),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(plugins::RedCenterPluginGroup)
        .run();
}
