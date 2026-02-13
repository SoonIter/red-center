use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubRound {
    SmallBlind,
    BigBlind,
    Boss,
}

impl SubRound {
    pub fn label(&self) -> &'static str {
        match self {
            SubRound::SmallBlind => "小盲注",
            SubRound::BigBlind => "大盲注",
            SubRound::Boss => "Boss",
        }
    }

    pub fn target_multiplier(&self) -> u32 {
        match self {
            SubRound::SmallBlind => 100,
            SubRound::BigBlind => 200,
            SubRound::Boss => 400,
        }
    }

    pub fn next(&self) -> Option<SubRound> {
        match self {
            SubRound::SmallBlind => Some(SubRound::BigBlind),
            SubRound::BigBlind => Some(SubRound::Boss),
            SubRound::Boss => None,
        }
    }
}

#[derive(Resource)]
pub struct GameState {
    pub level: u32,
    pub sub_round: SubRound,
    pub plays_remaining: u32,
    pub discards_remaining: u32,
    pub target_score: u32,
    pub current_score: u32,
    pub base_ante: u32,
    pub multiplier: u32,
    pub hand_size: usize,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            level: 1,
            sub_round: SubRound::SmallBlind,
            plays_remaining: 4,
            discards_remaining: 4,
            target_score: 100,
            current_score: 0,
            base_ante: 10,
            multiplier: 1,
            hand_size: 14,
        }
    }
}

impl GameState {
    pub fn reset_for_new_game(&mut self) {
        *self = Self::default();
    }

    pub fn reset_for_sub_round(&mut self) {
        self.plays_remaining = 4;
        self.discards_remaining = 4;
        self.current_score = 0;
        self.multiplier = 1;
        self.target_score = self.sub_round.target_multiplier() * self.level;
    }

    pub fn advance_sub_round(&mut self) -> bool {
        if let Some(next) = self.sub_round.next() {
            self.sub_round = next;
            true
        } else {
            // Passed boss → advance level
            self.level += 1;
            self.sub_round = SubRound::SmallBlind;
            true
        }
    }
}

#[derive(Resource, Default)]
pub struct TileWall {
    pub tiles: Vec<Entity>,
}

#[derive(Resource, Default)]
pub struct PlayerHand {
    pub tiles: Vec<Entity>,
}

#[derive(Resource, Default)]
pub struct PlayBoard {
    pub tiles: Vec<Entity>,
}
