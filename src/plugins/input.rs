use bevy::prelude::*;

use crate::components::tile::*;
use crate::plugins::game::AppState;
use crate::plugins::tile::{TILE_HEIGHT, TILE_WIDTH};
use crate::resources::PlayerHand;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            handle_tile_click.run_if(in_state(AppState::Playing)),
        );
    }
}

fn handle_tile_click(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    hand: Res<PlayerHand>,
    mut commands: Commands,
    tile_q: Query<(&Transform, Has<TileSelected>), With<Tile>>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_q.single() else {
        return;
    };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    // Check each hand tile for AABB collision
    for &entity in hand.tiles.iter() {
        if let Ok((transform, is_selected)) = tile_q.get(entity) {
            let tile_pos = transform.translation.truncate();
            let half_w = TILE_WIDTH / 2.0;
            let half_h = TILE_HEIGHT / 2.0;

            if world_pos.x >= tile_pos.x - half_w
                && world_pos.x <= tile_pos.x + half_w
                && world_pos.y >= tile_pos.y - half_h
                && world_pos.y <= tile_pos.y + half_h
            {
                if is_selected {
                    commands.entity(entity).remove::<TileSelected>();
                } else {
                    commands.entity(entity).insert(TileSelected);
                }
                break; // only toggle one tile per click
            }
        }
    }
}
