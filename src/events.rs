use bevy::prelude::*;

#[derive(Event, Clone)]
pub struct TileClickedEvent {
    pub tile_entity: Entity,
}

#[derive(Event, Clone)]
pub struct PlayTilesEvent;

#[derive(Event, Clone)]
pub struct DiscardTilesEvent;

#[derive(Event, Clone)]
pub struct ScoreCalculatedEvent {
    pub base: u32,
    pub multiplier: u32,
    pub total: u32,
    pub pattern_name: String,
}

#[derive(Event, Clone)]
pub struct RoundEndedEvent {
    pub passed: bool,
}

#[derive(Event, Clone)]
pub struct StartGameEvent;
