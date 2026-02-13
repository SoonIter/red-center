use bevy::prelude::*;
use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::components::tile::*;
use crate::plugins::game::AppState;
use crate::resources::*;

pub const TILE_WIDTH: f32 = 48.0;
pub const TILE_HEIGHT: f32 = 64.0;
pub const TILE_GAP: f32 = 4.0;

// Layout constants
// Window: 1280×720, Camera at (0,0) → visible Y range: [-360, 360]
// UI bottom section: 160px → maps to world Y [-360, -200]
// Center of bottom section: Y = -280, but selection area is upper part → Y ~ -260
pub const HAND_Y: f32 = -260.0;
// UI middle section: roughly Y [-200, 280] (joker area ~70px at top)
// Play area center: approximately Y = 40
pub const BOARD_Y: f32 = 40.0;
pub const BOARD_START_X: f32 = 0.0;

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileWall>()
            .init_resource::<PlayerHand>()
            .init_resource::<PlayBoard>()
            .add_systems(OnEnter(AppState::Playing), spawn_tiles)
            .add_systems(OnExit(AppState::Playing), cleanup_tiles)
            .add_systems(
                Update,
                update_tile_positions.run_if(in_state(AppState::Playing)),
            );
    }
}

#[derive(Component)]
pub struct TileText;

fn spawn_tiles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut wall: ResMut<TileWall>,
    mut hand: ResMut<PlayerHand>,
    mut play_board: ResMut<PlayBoard>,
    game_state: Option<Res<GameState>>,
) {
    let hand_size = game_state.map(|gs| gs.hand_size).unwrap_or(8);
    let font = asset_server.load("fonts/pixel.ttf");

    let mut tiles = Tile::generate_full_set();
    tiles.shuffle(&mut thread_rng());

    wall.tiles.clear();
    hand.tiles.clear();
    play_board.tiles.clear();

    for tile_data in tiles {
        let label = tile_data.id.label();
        let color = tile_data.id.suit_color();

        let entity = commands
            .spawn((
                tile_data,
                TileLocation::Wall,
                Sprite::from_color(
                    Color::srgb(0.95, 0.92, 0.85),
                    Vec2::new(TILE_WIDTH, TILE_HEIGHT),
                ),
                Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                Visibility::Hidden,
            ))
            .with_children(|parent| {
                parent.spawn((
                    TileText,
                    Text2d::new(label),
                    TextFont {
                        font: font.clone(),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(color),
                    Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
                ));
            })
            .id();

        wall.tiles.push(entity);
    }

    // Draw initial hand
    let draw_count = hand_size.min(wall.tiles.len());
    for _ in 0..draw_count {
        if let Some(entity) = wall.tiles.pop() {
            hand.tiles.push(entity);
        }
    }
}

fn update_tile_positions(
    hand: Res<PlayerHand>,
    play_board: Res<PlayBoard>,
    mut query: Query<(
        &mut Transform,
        &mut Visibility,
        &mut TileLocation,
        Has<TileSelected>,
    )>,
) {
    // Update hand tile positions
    let hand_count = hand.tiles.len();
    let hand_total_width = hand_count as f32 * (TILE_WIDTH + TILE_GAP) - TILE_GAP;
    let hand_start_x = -hand_total_width / 2.0 + TILE_WIDTH / 2.0;

    for (i, &entity) in hand.tiles.iter().enumerate() {
        if let Ok((mut transform, mut vis, mut loc, selected)) = query.get_mut(entity) {
            *loc = TileLocation::Hand;
            *vis = Visibility::Inherited;
            let x = hand_start_x + i as f32 * (TILE_WIDTH + TILE_GAP);
            let y_offset = if selected { 15.0 } else { 0.0 };
            transform.translation = Vec3::new(x, HAND_Y + y_offset, 2.0);
        }
    }

    // Update board tile positions
    let board_total_width = 14.0_f32 * (TILE_WIDTH + TILE_GAP) - TILE_GAP;
    let board_start_x = -board_total_width / 2.0 + TILE_WIDTH / 2.0 + BOARD_START_X;

    for (i, &entity) in play_board.tiles.iter().enumerate() {
        if let Ok((mut transform, mut vis, mut loc, _)) = query.get_mut(entity) {
            *loc = TileLocation::Board;
            *vis = Visibility::Inherited;
            let x = board_start_x + i as f32 * (TILE_WIDTH + TILE_GAP);
            transform.translation = Vec3::new(x, BOARD_Y, 2.0);
        }
    }

    // Hide tiles in wall / discarded
    // (wall tiles remain hidden by default since they spawn hidden)
}

fn cleanup_tiles(
    mut commands: Commands,
    tiles: Query<Entity, With<Tile>>,
    mut wall: ResMut<TileWall>,
    mut hand: ResMut<PlayerHand>,
    mut board: ResMut<PlayBoard>,
) {
    for entity in &tiles {
        commands.entity(entity).despawn();
    }
    wall.tiles.clear();
    hand.tiles.clear();
    board.tiles.clear();
}
