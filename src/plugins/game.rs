use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_sub_state::<PlayPhase>()
            .add_systems(Startup, setup_camera);
    }
}

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Menu,
    Playing,
    GameOver,
}

#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[source(AppState = AppState::Playing)]
pub enum PlayPhase {
    #[default]
    Selecting,
    Scoring,
    RoundResult,
}

#[derive(Component)]
pub struct MainCamera;

fn setup_camera(mut commands: Commands) {
    commands.insert_resource(ClearColor(Color::srgb(0.12, 0.12, 0.15)));
    commands.spawn((Camera2d, MainCamera, Msaa::Off));
}
