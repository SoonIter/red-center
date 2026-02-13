pub mod board;
pub mod game;
pub mod input;
pub mod scoring;
pub mod tile;
pub mod ui;

use bevy::app::{PluginGroup, PluginGroupBuilder};

pub struct RedCenterPluginGroup;

impl PluginGroup for RedCenterPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(game::GamePlugin)
            .add(tile::TilePlugin)
            .add(board::BoardPlugin)
            .add(input::InputPlugin)
            .add(scoring::ScoringPlugin)
            .add(ui::UiPlugin)
    }
}
