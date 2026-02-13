use bevy::prelude::*;

use crate::components::board::*;
use crate::events::*;
use crate::plugins::game::AppState;
use crate::resources::*;

const BG_DARK: Color = Color::srgb(0.12, 0.12, 0.15);
const BG_PANEL: Color = Color::srgb(0.18, 0.18, 0.22);
const BG_BUTTON: Color = Color::srgb(0.25, 0.25, 0.30);
const BG_BUTTON_HOVER: Color = Color::srgb(0.35, 0.35, 0.42);
const BG_BUTTON_PRESS: Color = Color::srgb(0.45, 0.55, 0.45);
const BORDER_COLOR: Color = Color::srgb(0.4, 0.4, 0.5);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.85);
const ACCENT_RED: Color = Color::srgb(0.85, 0.2, 0.2);

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Menu), setup_menu_ui)
            .add_systems(OnExit(AppState::Menu), cleanup::<MenuRoot>)
            .add_systems(OnEnter(AppState::Playing), setup_game_ui)
            .add_systems(OnExit(AppState::Playing), cleanup::<GameUiRoot>)
            .add_systems(OnEnter(AppState::GameOver), setup_gameover_ui)
            .add_systems(OnExit(AppState::GameOver), cleanup::<GameOverRoot>)
            .add_systems(
                Update,
                (
                    menu_button_system.run_if(in_state(AppState::Menu)),
                    (
                        game_button_system,
                        update_score_display,
                        update_wall_count,
                    )
                        .run_if(in_state(AppState::Playing)),
                    gameover_button_system.run_if(in_state(AppState::GameOver)),
                    button_hover_system,
                ),
            );
    }
}

// Root markers for cleanup
#[derive(Component)]
struct MenuRoot;

#[derive(Component)]
struct GameUiRoot;

#[derive(Component)]
struct GameOverRoot;

fn cleanup<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// ===================== MENU =====================

fn setup_menu_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/pixel.ttf");

    commands
        .spawn((
            MenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(40.0),
                ..default()
            },
            BackgroundColor(BG_DARK),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("红中"),
                TextFont {
                    font: font.clone(),
                    font_size: 72.0,
                    ..default()
                },
                TextColor(ACCENT_RED),
            ));

            // Subtitle
            parent.spawn((
                Text::new("Red Center"),
                TextFont {
                    font: font.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));

            // Start button
            parent
                .spawn((
                    StartButton,
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(60.0),
                        border: UiRect::all(Val::Px(3.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor::all(BORDER_COLOR),
                    BackgroundColor(BG_BUTTON),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("开始游戏"),
                        TextFont {
                            font: font.clone(),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));
                });
        });
}

fn menu_button_system(
    mut commands: Commands,
    query: Query<&Interaction, (Changed<Interaction>, With<StartButton>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            commands.insert_resource(GameState::default());
            next_state.set(AppState::Playing);
        }
    }
}

// ===================== GAME UI =====================

fn setup_game_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/pixel.ttf");

    commands
        .spawn((
            GameUiRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
        ))
        .with_children(|root| {
            // ---- Top: Joker area ----
            root.spawn((
                JokerArea,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(70.0),
                    border: UiRect::all(Val::Px(2.0)),
                    margin: UiRect::all(Val::Px(6.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BorderColor::all(BORDER_COLOR),
                BackgroundColor(BG_PANEL),
            ))
            .with_children(|joker| {
                joker.spawn((
                    Text::new("小丑区 (放置道具, 目前先空白)"),
                    TextFont {
                        font: font.clone(),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.5, 0.5, 0.55)),
                ));
            });

            // ---- Middle row ----
            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                padding: UiRect::horizontal(Val::Px(6.0)),
                column_gap: Val::Px(6.0),
                ..default()
            })
            .with_children(|mid| {
                // Left: Score area
                build_score_area(mid, &font);

                // Center: Play area (slots)
                build_play_area(mid, &font);

                // Right: Tile wall
                build_wall_display(mid, &font);
            });

            // ---- Bottom: Selection area + buttons ----
            root.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Px(160.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(8.0),
                padding: UiRect::bottom(Val::Px(10.0)),
                ..default()
            })
            .with_children(|bottom| {
                // Selection area (transparent so world-space tile sprites show through)
                bottom.spawn((
                    SelectionArea,
                    Node {
                        width: Val::Px(600.0),
                        height: Val::Px(80.0),
                        border: UiRect::all(Val::Px(2.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BorderColor::all(BORDER_COLOR),
                    // No BackgroundColor — transparent so sprites show through
                ));

                // Button row
                bottom
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(20.0),
                        ..default()
                    })
                    .with_children(|btn_row| {
                        spawn_game_button(btn_row, &font, "菜单", MenuButton);
                        spawn_game_button(btn_row, &font, "出牌", PlayButton);
                        spawn_game_button(btn_row, &font, "弃牌", DiscardButton);
                    });
            });
        });
}

fn build_score_area(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn((
            ScoreArea,
            Node {
                width: Val::Px(180.0),
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(2.0)),
                padding: UiRect::all(Val::Px(10.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            BorderColor::all(BORDER_COLOR),
            BackgroundColor(BG_PANEL),
        ))
        .with_children(|score| {
            // Sub round label
            score.spawn((
                SubRoundText,
                Text::new("小盲注"),
                TextFont {
                    font: font.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(ACCENT_RED),
            ));

            // Base score
            score.spawn((
                BaseScoreText,
                Text::new("底注: 10"),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));

            // Multiplier
            score.spawn((
                MultiplierText,
                Text::new("× 倍率: 1"),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));

            // Total
            score.spawn((
                TotalScoreText,
                Text::new("= 0"),
                TextFont {
                    font: font.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.2)),
            ));

            // Target
            score.spawn((
                TargetScoreText,
                Text::new("目标: 100"),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.8, 1.0)),
            ));

            // Plays remaining
            score.spawn((
                PlaysRemainingText,
                Text::new("出牌: 4"),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));

            // Discard remaining
            score.spawn((
                DiscardRemainingText,
                Text::new("弃牌: 4"),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));

            // Hand pattern
            score.spawn((
                HandPatternText,
                Text::new(""),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 1.0, 0.5)),
            ));
        });
}

fn build_play_area(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn((
            PlayArea,
            Node {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(2.0)),
                padding: UiRect::all(Val::Px(10.0)),
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(BORDER_COLOR),
            // No BackgroundColor — transparent so world-space board tile sprites show through
        ))
        .with_children(|play| {
            // Label
            play.spawn((
                Text::new("出牌区"),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.55)),
            ));

            // 14 empty slots (semi-transparent so board sprites can show through)
            play.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                margin: UiRect::top(Val::Px(10.0)),
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_children(|slots| {
                for _ in 0..14 {
                    slots.spawn((
                        Node {
                            width: Val::Px(44.0),
                            height: Val::Px(60.0),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BorderColor::all(Color::srgb(0.3, 0.3, 0.35)),
                        BackgroundColor(Color::srgba(0.14, 0.14, 0.17, 0.3)),
                    ));
                }
            });
        });
}

fn build_wall_display(parent: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    parent
        .spawn((
            TileWallDisplay,
            Node {
                width: Val::Px(100.0),
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(2.0)),
                padding: UiRect::all(Val::Px(10.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(8.0),
                ..default()
            },
            BorderColor::all(BORDER_COLOR),
            BackgroundColor(BG_PANEL),
        ))
        .with_children(|wall| {
            wall.spawn((
                Text::new("牌山"),
                TextFont {
                    font: font.clone(),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));
            wall.spawn((
                WallCountText,
                Text::new("128 张"),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.75)),
            ));
        });
}

fn spawn_game_button(parent: &mut ChildSpawnerCommands, font: &Handle<Font>, label: &str, marker: impl Component) {
    parent
        .spawn((
            marker,
            Button,
            Node {
                width: Val::Px(100.0),
                height: Val::Px(40.0),
                border: UiRect::all(Val::Px(2.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(BORDER_COLOR),
            BackgroundColor(BG_BUTTON),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));
        });
}

// ===================== GAME BUTTON HANDLING =====================

fn game_button_system(
    mut commands: Commands,
    play_q: Query<&Interaction, (Changed<Interaction>, With<PlayButton>)>,
    discard_q: Query<&Interaction, (Changed<Interaction>, With<DiscardButton>)>,
    menu_q: Query<&Interaction, (Changed<Interaction>, With<MenuButton>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in &play_q {
        if *interaction == Interaction::Pressed {
            commands.trigger(PlayTilesEvent);
        }
    }
    for interaction in &discard_q {
        if *interaction == Interaction::Pressed {
            commands.trigger(DiscardTilesEvent);
        }
    }
    for interaction in &menu_q {
        if *interaction == Interaction::Pressed {
            next_state.set(AppState::Menu);
        }
    }
}

// ===================== DISPLAY UPDATES =====================

fn update_score_display(
    game_state: Option<Res<GameState>>,
    mut base_q: Query<&mut Text, (With<BaseScoreText>, Without<MultiplierText>, Without<TotalScoreText>, Without<TargetScoreText>, Without<PlaysRemainingText>, Without<DiscardRemainingText>, Without<SubRoundText>)>,
    mut mult_q: Query<&mut Text, (With<MultiplierText>, Without<BaseScoreText>, Without<TotalScoreText>, Without<TargetScoreText>, Without<PlaysRemainingText>, Without<DiscardRemainingText>, Without<SubRoundText>)>,
    mut total_q: Query<&mut Text, (With<TotalScoreText>, Without<BaseScoreText>, Without<MultiplierText>, Without<TargetScoreText>, Without<PlaysRemainingText>, Without<DiscardRemainingText>, Without<SubRoundText>)>,
    mut target_q: Query<&mut Text, (With<TargetScoreText>, Without<BaseScoreText>, Without<MultiplierText>, Without<TotalScoreText>, Without<PlaysRemainingText>, Without<DiscardRemainingText>, Without<SubRoundText>)>,
    mut plays_q: Query<&mut Text, (With<PlaysRemainingText>, Without<BaseScoreText>, Without<MultiplierText>, Without<TotalScoreText>, Without<TargetScoreText>, Without<DiscardRemainingText>, Without<SubRoundText>)>,
    mut discard_q: Query<&mut Text, (With<DiscardRemainingText>, Without<BaseScoreText>, Without<MultiplierText>, Without<TotalScoreText>, Without<TargetScoreText>, Without<PlaysRemainingText>, Without<SubRoundText>)>,
    mut sub_round_q: Query<&mut Text, (With<SubRoundText>, Without<BaseScoreText>, Without<MultiplierText>, Without<TotalScoreText>, Without<TargetScoreText>, Without<PlaysRemainingText>, Without<DiscardRemainingText>)>,
) {
    let Some(gs) = game_state else { return };
    if !gs.is_changed() {
        return;
    }

    if let Ok(mut text) = base_q.single_mut() {
        text.0 = format!("底注: {}", gs.base_ante);
    }
    if let Ok(mut text) = mult_q.single_mut() {
        text.0 = format!("× 倍率: {}", gs.multiplier);
    }
    if let Ok(mut text) = total_q.single_mut() {
        text.0 = format!("= {}", gs.current_score);
    }
    if let Ok(mut text) = target_q.single_mut() {
        text.0 = format!("目标: {}", gs.target_score);
    }
    if let Ok(mut text) = plays_q.single_mut() {
        text.0 = format!("出牌: {}", gs.plays_remaining);
    }
    if let Ok(mut text) = discard_q.single_mut() {
        text.0 = format!("弃牌: {}", gs.discards_remaining);
    }
    if let Ok(mut text) = sub_round_q.single_mut() {
        text.0 = format!("Lv.{} {}", gs.level, gs.sub_round.label());
    }
}

fn update_wall_count(
    wall: Res<TileWall>,
    mut query: Query<&mut Text, With<WallCountText>>,
) {
    if !wall.is_changed() {
        return;
    }
    if let Ok(mut text) = query.single_mut() {
        text.0 = format!("{} 张", wall.tiles.len());
    }
}

// ===================== GAME OVER =====================

fn setup_gameover_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_state: Option<Res<GameState>>,
) {
    let font = asset_server.load("fonts/pixel.ttf");
    let level = game_state.map(|gs| gs.level).unwrap_or(1);

    commands
        .spawn((
            GameOverRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(30.0),
                ..default()
            },
            BackgroundColor(BG_DARK),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("游戏结束"),
                TextFont {
                    font: font.clone(),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(ACCENT_RED),
            ));

            parent.spawn((
                Text::new(format!("到达关卡: {}", level)),
                TextFont {
                    font: font.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));

            parent
                .spawn((
                    StartButton,
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(60.0),
                        border: UiRect::all(Val::Px(3.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor::all(BORDER_COLOR),
                    BackgroundColor(BG_BUTTON),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("重新开始"),
                        TextFont {
                            font: font.clone(),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));
                });
        });
}

fn gameover_button_system(
    mut commands: Commands,
    query: Query<&Interaction, (Changed<Interaction>, With<StartButton>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            commands.insert_resource(GameState::default());
            next_state.set(AppState::Menu);
        }
    }
}

// ===================== BUTTON HOVER =====================

fn button_hover_system(
    mut query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, mut bg) in &mut query {
        match *interaction {
            Interaction::Pressed => *bg = BackgroundColor(BG_BUTTON_PRESS),
            Interaction::Hovered => *bg = BackgroundColor(BG_BUTTON_HOVER),
            Interaction::None => *bg = BackgroundColor(BG_BUTTON),
        }
    }
}
