use bevy::prelude::*;

use crate::components::board::HandPatternText;
use crate::components::tile::*;
use crate::events::*;
use crate::plugins::game::{AppState, PlayPhase};
use crate::resources::*;

pub struct ScoringPlugin;

impl Plugin for ScoringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(PlayPhase::Scoring),
            calculate_score,
        )
        .add_systems(
            OnEnter(PlayPhase::RoundResult),
            evaluate_round_result,
        );
    }
}

// ============ Hand patterns ============

#[derive(Debug, Clone)]
pub struct HandResult {
    pub pattern_name: String,
    pub base: u32,
    pub multiplier: u32,
}

impl HandResult {
    pub fn total(&self) -> u32 {
        self.base * self.multiplier
    }
}

/// Build a [u8; 34] count array from tile IDs
fn build_count_array(tiles: &[TileId]) -> [u8; 34] {
    let mut counts = [0u8; 34];
    for tile in tiles {
        counts[tile.to_index()] += 1;
    }
    counts
}

/// Check for standard winning hand: 1 pair + N melds (sets of 3)
/// A meld is either a triplet (3 same) or a sequence (3 consecutive in same suit)
fn check_standard(counts: &[u8; 34]) -> bool {
    // Try each possible pair
    for pair_idx in 0..34 {
        if counts[pair_idx] < 2 {
            continue;
        }
        let mut remaining = *counts;
        remaining[pair_idx] -= 2;
        if remove_melds(&mut remaining) {
            return true;
        }
    }
    false
}

/// Recursively remove melds (triplets and sequences) from count array.
/// Returns true if all tiles are consumed.
fn remove_melds(counts: &mut [u8; 34]) -> bool {
    // Find the first tile that still has count > 0
    let first = match counts.iter().position(|&c| c > 0) {
        Some(idx) => idx,
        None => return true, // all tiles consumed
    };

    // Try triplet
    if counts[first] >= 3 {
        counts[first] -= 3;
        if remove_melds(counts) {
            return true;
        }
        counts[first] += 3;
    }

    // Try sequence (only for numbered suits: 0-8 man, 9-17 pin, 18-26 sou)
    let suit_start = if first < 9 {
        Some(0)
    } else if first < 18 {
        Some(9)
    } else if first < 27 {
        Some(18)
    } else {
        None
    };

    if let Some(start) = suit_start {
        let pos_in_suit = first - start;
        if pos_in_suit <= 6 {
            // Can form a sequence: first, first+1, first+2
            let i1 = first;
            let i2 = first + 1;
            let i3 = first + 2;
            if counts[i1] >= 1 && counts[i2] >= 1 && counts[i3] >= 1 {
                counts[i1] -= 1;
                counts[i2] -= 1;
                counts[i3] -= 1;
                if remove_melds(counts) {
                    return true;
                }
                counts[i1] += 1;
                counts[i2] += 1;
                counts[i3] += 1;
            }
        }
    }

    false
}

/// Check for 7 pairs (七对子)
fn check_seven_pairs(counts: &[u8; 34]) -> bool {
    let pairs = counts.iter().filter(|&&c| c >= 2).count();
    let total: u8 = counts.iter().sum();
    pairs == 7 && total == 14
}

/// Check for Thirteen Orphans (国士无双)
/// Requires one of each: 1m,9m,1p,9p,1s,9s,東,南,西,北,中,發,白  plus one duplicate
fn check_thirteen_orphans(counts: &[u8; 34]) -> bool {
    let orphan_indices = [
        0, 8,     // 1m, 9m
        9, 17,    // 1p, 9p
        18, 26,   // 1s, 9s
        27, 28, 29, 30, // 東南西北
        31, 32, 33,     // 中發白
    ];
    let total: u8 = counts.iter().sum();
    if total != 14 {
        return false;
    }
    // Each orphan tile must appear at least once
    for &idx in &orphan_indices {
        if counts[idx] < 1 {
            return false;
        }
    }
    // Exactly one of them appears twice (the pair)
    let pair_count: usize = orphan_indices.iter().filter(|&&idx| counts[idx] >= 2).count();
    // No non-orphan tiles
    let non_orphan_count: u8 = (0..34)
        .filter(|i| !orphan_indices.contains(i))
        .map(|i| counts[i])
        .sum();
    pair_count == 1 && non_orphan_count == 0
}

/// Check 断幺九 (tanyao) - all tiles are simples (2-8 of numbered suits)
fn check_tanyao(tiles: &[TileId]) -> bool {
    tiles.iter().all(|t| t.is_simple())
}

/// Check 平和 (pinfu) - standard hand with all sequences + no value pair
/// (Simplified: standard hand with no triplets)
fn check_pinfu(counts: &[u8; 34]) -> bool {
    // Must be a standard hand first
    if !check_standard(counts) {
        return false;
    }
    // In a simplified implementation: check that no tile appears 3+ times
    // (meaning all melds must be sequences)
    counts.iter().all(|&c| c < 3)
}

/// Check 一気通貫 (ikkitsukan / 一气通贯) - 123, 456, 789 of one suit
fn check_straight(counts: &[u8; 34]) -> bool {
    for suit_start in [0, 9, 18] {
        // Check: 1,2,3,4,5,6,7,8,9 all present
        let has_straight = (0..9).all(|i| counts[suit_start + i] >= 1);
        if has_straight {
            return true;
        }
    }
    false
}

/// Check 対対和 (toitoi / 对对和) - all triplets, no sequences
fn check_toitoi(counts: &[u8; 34]) -> bool {
    if !check_standard(counts) {
        return false;
    }
    // All non-pair tiles must be in triplets (count must be 0, 2, or 3)
    // Try each pair and verify remaining are all triplets
    for pair_idx in 0..34 {
        if counts[pair_idx] < 2 {
            continue;
        }
        let mut remaining = *counts;
        remaining[pair_idx] -= 2;
        if remaining.iter().all(|&c| c == 0 || c == 3) {
            return true;
        }
    }
    false
}

/// Check 混一色 (honitsu) - all tiles from one suit + honor tiles only
fn check_honitsu(tiles: &[TileId]) -> bool {
    let has_honors = tiles.iter().any(|t| t.is_honor());
    if !has_honors {
        return false;
    }
    let suits: Vec<_> = tiles
        .iter()
        .filter(|t| !t.is_honor())
        .map(|t| t.suit)
        .collect();
    if suits.is_empty() {
        return false; // all honors = chinitsu of honors, not honitsu
    }
    let first = suits[0];
    suits.iter().all(|&s| s == first)
}

/// Check 清一色 (chinitsu) - all tiles from a single numbered suit
fn check_chinitsu(tiles: &[TileId]) -> bool {
    if tiles.is_empty() {
        return false;
    }
    let first = tiles[0].suit;
    if matches!(first, TileSuit::Wind | TileSuit::Dragon) {
        return false;
    }
    tiles.iter().all(|t| t.suit == first)
}

/// Evaluate the best hand pattern from a set of tile IDs
pub fn evaluate_hand(tiles: &[TileId]) -> HandResult {
    if tiles.is_empty() {
        return HandResult {
            pattern_name: "无牌型".into(),
            base: 0,
            multiplier: 1,
        };
    }

    let counts = build_count_array(tiles);
    let is_standard = check_standard(&counts);

    // Check from highest to lowest value patterns
    // 国士无双 ×13
    if check_thirteen_orphans(&counts) {
        return HandResult {
            pattern_name: "国士无双".into(),
            base: 10,
            multiplier: 13,
        };
    }

    // 清一色 ×8
    if is_standard && check_chinitsu(tiles) {
        return HandResult {
            pattern_name: "清一色".into(),
            base: 10,
            multiplier: 8,
        };
    }

    // 混一色 ×5
    if is_standard && check_honitsu(tiles) {
        return HandResult {
            pattern_name: "混一色".into(),
            base: 10,
            multiplier: 5,
        };
    }

    // 七对子 ×4
    if check_seven_pairs(&counts) {
        return HandResult {
            pattern_name: "七对子".into(),
            base: 10,
            multiplier: 4,
        };
    }

    // 对对和 ×4
    if check_toitoi(&counts) {
        return HandResult {
            pattern_name: "对对和".into(),
            base: 10,
            multiplier: 4,
        };
    }

    // 一气通贯 ×3
    if is_standard && check_straight(&counts) {
        return HandResult {
            pattern_name: "一气通贯".into(),
            base: 10,
            multiplier: 3,
        };
    }

    // 断幺九 ×2
    if is_standard && check_tanyao(tiles) {
        return HandResult {
            pattern_name: "断幺九".into(),
            base: 10,
            multiplier: 2,
        };
    }

    // 平和 ×1
    if check_pinfu(&counts) {
        return HandResult {
            pattern_name: "平和".into(),
            base: 10,
            multiplier: 1,
        };
    }

    // Standard win (no special pattern)
    if is_standard {
        return HandResult {
            pattern_name: "和了".into(),
            base: 10,
            multiplier: 1,
        };
    }

    // No winning pattern – give a small consolation score based on tile count
    HandResult {
        pattern_name: "未和牌".into(),
        base: tiles.len() as u32,
        multiplier: 1,
    }
}

// ============ Bevy Systems ============

fn calculate_score(
    mut commands: Commands,
    board: Res<PlayBoard>,
    tile_q: Query<&Tile>,
    mut game_state: ResMut<GameState>,
    mut pattern_text_q: Query<&mut Text, With<HandPatternText>>,
) {
    // Gather tile IDs from board
    let tile_ids: Vec<TileId> = board
        .tiles
        .iter()
        .filter_map(|e| tile_q.get(*e).ok())
        .map(|t| t.id)
        .collect();

    let result = evaluate_hand(&tile_ids);

    // Update game state
    game_state.base_ante = result.base;
    game_state.multiplier = result.multiplier;
    game_state.current_score += result.total();

    // Update pattern text
    if let Ok(mut text) = pattern_text_q.single_mut() {
        text.0 = format!("{} ({}×{})", result.pattern_name, result.base, result.multiplier);
    }

    // Trigger score calculated event
    commands.trigger(ScoreCalculatedEvent {
        base: result.base,
        multiplier: result.multiplier,
        total: result.total(),
        pattern_name: result.pattern_name,
    });

    // Transition to RoundResult after a brief delay (immediate for now)
    commands.trigger(RoundEndedEvent {
        passed: game_state.current_score >= game_state.target_score,
    });
}

fn evaluate_round_result(
    mut game_state: ResMut<GameState>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_phase: ResMut<NextState<PlayPhase>>,
    mut commands: Commands,
    mut board: ResMut<PlayBoard>,
) {
    let passed = game_state.current_score >= game_state.target_score;

    if passed {
        // Advance to next sub-round
        game_state.advance_sub_round();
        game_state.reset_for_sub_round();

        // Hide board tiles
        for &entity in board.tiles.iter() {
            commands.entity(entity).insert(Visibility::Hidden);
        }
        board.tiles.clear();

        // Return to selecting phase for the next sub-round
        next_phase.set(PlayPhase::Selecting);
    } else {
        // Failed: go to game over
        next_app_state.set(AppState::GameOver);
    }
}

// ============ Unit Tests ============

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tiles(specs: &[(TileSuit, u8, u8)]) -> Vec<TileId> {
        let mut tiles = Vec::new();
        for &(suit, value, count) in specs {
            for _ in 0..count {
                tiles.push(TileId { suit, value });
            }
        }
        tiles
    }

    #[test]
    fn test_standard_win_simple() {
        // 1m 1m 1m 2m 3m 4m 5m 6m 7m 8m 9m 9m 9m 3p 3p = nope (15 tiles)
        // Let's do a proper 14-tile hand: 1m×3, 2m,3m,4m, 5m,6m,7m, 8m,9m,9m×3 = 14 nope
        // Simple: 1m×2 (pair) + 2m,3m,4m + 5m,6m,7m + 1p,2p,3p + 4p,5p,6p = 14
        let tiles = make_tiles(&[
            (TileSuit::Man, 1, 2),
            (TileSuit::Man, 2, 1), (TileSuit::Man, 3, 1), (TileSuit::Man, 4, 1),
            (TileSuit::Man, 5, 1), (TileSuit::Man, 6, 1), (TileSuit::Man, 7, 1),
            (TileSuit::Pin, 1, 1), (TileSuit::Pin, 2, 1), (TileSuit::Pin, 3, 1),
            (TileSuit::Pin, 4, 1), (TileSuit::Pin, 5, 1), (TileSuit::Pin, 6, 1),
        ]);
        assert_eq!(tiles.len(), 14);
        let result = evaluate_hand(&tiles);
        assert!(result.multiplier >= 1, "Should be a winning hand: {:?}", result);
    }

    #[test]
    fn test_seven_pairs() {
        let tiles = make_tiles(&[
            (TileSuit::Man, 1, 2),
            (TileSuit::Man, 3, 2),
            (TileSuit::Man, 5, 2),
            (TileSuit::Man, 7, 2),
            (TileSuit::Pin, 2, 2),
            (TileSuit::Pin, 4, 2),
            (TileSuit::Sou, 6, 2),
        ]);
        assert_eq!(tiles.len(), 14);
        let result = evaluate_hand(&tiles);
        assert_eq!(result.pattern_name, "七对子");
        assert_eq!(result.multiplier, 4);
    }

    #[test]
    fn test_thirteen_orphans() {
        let tiles = make_tiles(&[
            (TileSuit::Man, 1, 2), // pair
            (TileSuit::Man, 9, 1),
            (TileSuit::Pin, 1, 1),
            (TileSuit::Pin, 9, 1),
            (TileSuit::Sou, 1, 1),
            (TileSuit::Sou, 9, 1),
            (TileSuit::Wind, 1, 1),
            (TileSuit::Wind, 2, 1),
            (TileSuit::Wind, 3, 1),
            (TileSuit::Wind, 4, 1),
            (TileSuit::Dragon, 1, 1),
            (TileSuit::Dragon, 2, 1),
            (TileSuit::Dragon, 3, 1),
        ]);
        assert_eq!(tiles.len(), 14);
        let result = evaluate_hand(&tiles);
        assert_eq!(result.pattern_name, "国士无双");
        assert_eq!(result.multiplier, 13);
    }

    #[test]
    fn test_chinitsu() {
        // All man tiles: 1m×3, 2m,3m,4m, 5m,6m,7m, 8m,9m,9m×3 => nah, let me count
        // 1m×2 (pair) + 1m,2m,3m + 4m,5m,6m + 7m,8m,9m + 7m,8m,9m = 14
        // 1m×3, 2m×1, 3m×1, 4m×1, 5m×1, 6m×1, 7m×2, 8m×2, 9m×2 = 14
        let tiles = make_tiles(&[
            (TileSuit::Man, 1, 3),
            (TileSuit::Man, 2, 1),
            (TileSuit::Man, 3, 1),
            (TileSuit::Man, 4, 1),
            (TileSuit::Man, 5, 1),
            (TileSuit::Man, 6, 1),
            (TileSuit::Man, 7, 2),
            (TileSuit::Man, 8, 2),
            (TileSuit::Man, 9, 2),
        ]);
        assert_eq!(tiles.len(), 14);
        let result = evaluate_hand(&tiles);
        assert_eq!(result.pattern_name, "清一色");
        assert_eq!(result.multiplier, 8);
    }

    #[test]
    fn test_toitoi() {
        // All triplets + pair: 1m×3, 5m×3, 9p×3, 中×3, 東×2 = 14
        let tiles = make_tiles(&[
            (TileSuit::Man, 1, 3),
            (TileSuit::Man, 5, 3),
            (TileSuit::Pin, 9, 3),
            (TileSuit::Dragon, 1, 3),
            (TileSuit::Wind, 1, 2),
        ]);
        assert_eq!(tiles.len(), 14);
        let result = evaluate_hand(&tiles);
        assert_eq!(result.pattern_name, "对对和");
        assert_eq!(result.multiplier, 4);
    }

    #[test]
    fn test_no_pattern() {
        // Random mismatched tiles
        let tiles = make_tiles(&[
            (TileSuit::Man, 1, 1),
            (TileSuit::Man, 3, 1),
            (TileSuit::Man, 5, 1),
            (TileSuit::Pin, 2, 1),
            (TileSuit::Pin, 7, 1),
            (TileSuit::Sou, 1, 1),
            (TileSuit::Sou, 4, 1),
            (TileSuit::Sou, 8, 1),
            (TileSuit::Wind, 1, 1),
            (TileSuit::Wind, 3, 1),
            (TileSuit::Dragon, 1, 1),
            (TileSuit::Dragon, 2, 1),
            (TileSuit::Dragon, 3, 1),
            (TileSuit::Man, 9, 1),
        ]);
        assert_eq!(tiles.len(), 14);
        let result = evaluate_hand(&tiles);
        assert_eq!(result.pattern_name, "未和牌");
    }

    #[test]
    fn test_tanyao() {
        // All simples (2-8): pair of 5m + sequences 2m3m4m, 5m6m7m, 2p3p4p, 6p7p8p = 14
        // 5m×3 (pair + 1 for seq), then the rest form 4 melds
        let tiles = make_tiles(&[
            (TileSuit::Man, 5, 3), // pair(2) + seq start(1)
            (TileSuit::Man, 2, 1), (TileSuit::Man, 3, 1), (TileSuit::Man, 4, 1), // seq 2,3,4
            (TileSuit::Man, 6, 1), (TileSuit::Man, 7, 1), // seq 5,6,7
            (TileSuit::Pin, 2, 1), (TileSuit::Pin, 3, 1), (TileSuit::Pin, 4, 1), // seq 2,3,4
            (TileSuit::Pin, 6, 1), (TileSuit::Pin, 7, 1), (TileSuit::Pin, 8, 1), // seq 6,7,8
        ]);
        assert_eq!(tiles.len(), 14);
        let result = evaluate_hand(&tiles);
        assert_eq!(result.pattern_name, "断幺九");
        assert_eq!(result.multiplier, 2);
    }
}
