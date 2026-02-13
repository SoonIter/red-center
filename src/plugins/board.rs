use bevy::prelude::*;

use crate::components::tile::*;
use crate::events::*;
use crate::plugins::game::{AppState, PlayPhase};
use crate::resources::*;

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_play_tiles)
            .add_observer(on_discard_tiles)
            .add_systems(
                Update,
                check_phase_transition
                    .run_if(in_state(AppState::Playing))
                    .run_if(in_state(PlayPhase::Selecting)),
            );
    }
}

/// Observer: when PlayTilesEvent is triggered, move selected tiles from hand to board
fn on_play_tiles(
    _trigger: On<PlayTilesEvent>,
    mut commands: Commands,
    mut hand: ResMut<PlayerHand>,
    mut board: ResMut<PlayBoard>,
    mut wall: ResMut<TileWall>,
    mut game_state: ResMut<GameState>,
    selected_q: Query<Entity, (With<Tile>, With<TileSelected>)>,
) {
    if game_state.plays_remaining == 0 {
        return;
    }

    // Collect selected tile entities that are in the hand
    let selected_in_hand: Vec<Entity> = hand
        .tiles
        .iter()
        .copied()
        .filter(|e| selected_q.contains(*e))
        .collect();

    if selected_in_hand.is_empty() {
        return;
    }

    // Move tiles from hand to board
    for &entity in &selected_in_hand {
        hand.tiles.retain(|e| *e != entity);
        board.tiles.push(entity);
        commands.entity(entity).remove::<TileSelected>();
    }

    // Draw replacement tiles from wall
    let draw_count = selected_in_hand.len().min(wall.tiles.len());
    for _ in 0..draw_count {
        if let Some(entity) = wall.tiles.pop() {
            hand.tiles.push(entity);
        }
    }

    game_state.plays_remaining = game_state.plays_remaining.saturating_sub(1);
}

/// Observer: when DiscardTilesEvent is triggered, discard selected tiles from hand
fn on_discard_tiles(
    _trigger: On<DiscardTilesEvent>,
    mut commands: Commands,
    mut hand: ResMut<PlayerHand>,
    mut wall: ResMut<TileWall>,
    mut game_state: ResMut<GameState>,
    selected_q: Query<Entity, (With<Tile>, With<TileSelected>)>,
) {
    if game_state.discards_remaining == 0 {
        return;
    }

    let selected_in_hand: Vec<Entity> = hand
        .tiles
        .iter()
        .copied()
        .filter(|e| selected_q.contains(*e))
        .collect();

    if selected_in_hand.is_empty() {
        return;
    }

    // Remove from hand, mark as discarded, hide
    for &entity in &selected_in_hand {
        hand.tiles.retain(|e| *e != entity);
        commands
            .entity(entity)
            .remove::<TileSelected>()
            .insert(Visibility::Hidden);
    }

    // Draw replacement tiles from wall
    let draw_count = selected_in_hand.len().min(wall.tiles.len());
    for _ in 0..draw_count {
        if let Some(entity) = wall.tiles.pop() {
            hand.tiles.push(entity);
        }
    }

    game_state.discards_remaining = game_state.discards_remaining.saturating_sub(1);
}

/// Check if the board is full (14 tiles) or plays are exhausted → transition to Scoring
fn check_phase_transition(
    board: Res<PlayBoard>,
    game_state: Res<GameState>,
    mut next_phase: ResMut<NextState<PlayPhase>>,
) {
    if board.tiles.len() >= 14 || game_state.plays_remaining == 0 {
        next_phase.set(PlayPhase::Scoring);
    }
}
