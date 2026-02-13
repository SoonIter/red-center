use bevy::prelude::*;

// UI area marker components
#[derive(Component)]
pub struct JokerArea;

#[derive(Component)]
pub struct ScoreArea;

#[derive(Component)]
pub struct PlayArea;

#[derive(Component)]
pub struct SelectionArea;

#[derive(Component)]
pub struct TileWallDisplay;

// Button markers
#[derive(Component)]
pub struct MenuButton;

#[derive(Component)]
pub struct PlayButton;

#[derive(Component)]
pub struct DiscardButton;

#[derive(Component)]
pub struct StartButton;

// Text display markers
#[derive(Component)]
pub struct BaseScoreText;

#[derive(Component)]
pub struct MultiplierText;

#[derive(Component)]
pub struct TotalScoreText;

#[derive(Component)]
pub struct TargetScoreText;

#[derive(Component)]
pub struct WallCountText;

#[derive(Component)]
pub struct PlaysRemainingText;

#[derive(Component)]
pub struct DiscardRemainingText;

#[derive(Component)]
pub struct SubRoundText;

#[derive(Component)]
pub struct HandPatternText;
