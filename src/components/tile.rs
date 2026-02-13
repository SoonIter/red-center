use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TileSuit {
    Man,    // 万
    Pin,    // 筒
    Sou,    // 条
    Wind,   // 风
    Dragon, // 箭 (中发白)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TileId {
    pub suit: TileSuit,
    pub value: u8,
}

impl TileId {
    pub fn label(&self) -> String {
        match self.suit {
            TileSuit::Man => format!("{}万", self.value),
            TileSuit::Pin => format!("{}筒", self.value),
            TileSuit::Sou => format!("{}条", self.value),
            TileSuit::Wind => match self.value {
                1 => "东".into(),
                2 => "南".into(),
                3 => "西".into(),
                4 => "北".into(),
                _ => unreachable!(),
            },
            TileSuit::Dragon => match self.value {
                1 => "中".into(),
                2 => "发".into(),
                3 => "白".into(),
                _ => unreachable!(),
            },
        }
    }

    pub fn suit_color(&self) -> Color {
        match self.suit {
            TileSuit::Man => Color::srgb(0.8, 0.2, 0.2),    // red
            TileSuit::Pin => Color::srgb(0.2, 0.5, 0.8),    // blue
            TileSuit::Sou => Color::srgb(0.2, 0.7, 0.3),    // green
            TileSuit::Wind => Color::srgb(0.3, 0.3, 0.3),   // dark gray
            TileSuit::Dragon => match self.value {
                1 => Color::srgb(0.9, 0.1, 0.1), // 中 = red
                2 => Color::srgb(0.1, 0.7, 0.2), // 发 = green
                3 => Color::srgb(0.5, 0.5, 0.6), // 白 = gray-white
                _ => unreachable!(),
            },
        }
    }

    pub fn is_honor(&self) -> bool {
        matches!(self.suit, TileSuit::Wind | TileSuit::Dragon)
    }

    pub fn is_simple(&self) -> bool {
        !self.is_honor() && self.value >= 2 && self.value <= 8
    }

    pub fn is_terminal(&self) -> bool {
        !self.is_honor() && (self.value == 1 || self.value == 9)
    }

    /// Convert to index in [u8; 34] counting array
    pub fn to_index(&self) -> usize {
        match self.suit {
            TileSuit::Man => (self.value - 1) as usize,
            TileSuit::Pin => 9 + (self.value - 1) as usize,
            TileSuit::Sou => 18 + (self.value - 1) as usize,
            TileSuit::Wind => 27 + (self.value - 1) as usize,
            TileSuit::Dragon => 31 + (self.value - 1) as usize,
        }
    }

    /// Convert from index in [u8; 34] counting array
    pub fn from_index(index: usize) -> Self {
        match index {
            0..=8 => TileId { suit: TileSuit::Man, value: (index + 1) as u8 },
            9..=17 => TileId { suit: TileSuit::Pin, value: (index - 8) as u8 },
            18..=26 => TileId { suit: TileSuit::Sou, value: (index - 17) as u8 },
            27..=30 => TileId { suit: TileSuit::Wind, value: (index - 26) as u8 },
            31..=33 => TileId { suit: TileSuit::Dragon, value: (index - 30) as u8 },
            _ => unreachable!(),
        }
    }
}

impl PartialOrd for TileId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TileId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.suit.cmp(&other.suit).then(self.value.cmp(&other.value))
    }
}

#[derive(Component, Debug, Clone)]
pub struct Tile {
    pub id: TileId,
    pub copy_index: u8,
}

impl Tile {
    pub fn generate_full_set() -> Vec<Tile> {
        let mut tiles = Vec::with_capacity(136);
        // 万/筒/条: 各 1-9, 每种 4 张
        for suit in [TileSuit::Man, TileSuit::Pin, TileSuit::Sou] {
            for value in 1..=9 {
                for copy in 0..4 {
                    tiles.push(Tile {
                        id: TileId { suit, value },
                        copy_index: copy,
                    });
                }
            }
        }
        // 风: 东南西北, 每种 4 张
        for value in 1..=4 {
            for copy in 0..4 {
                tiles.push(Tile {
                    id: TileId { suit: TileSuit::Wind, value },
                    copy_index: copy,
                });
            }
        }
        // 箭: 中发白, 每种 4 张
        for value in 1..=3 {
            for copy in 0..4 {
                tiles.push(Tile {
                    id: TileId { suit: TileSuit::Dragon, value },
                    copy_index: copy,
                });
            }
        }
        tiles
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TileLocation {
    #[default]
    Wall,
    Hand,
    Board,
    Discarded,
}

#[derive(Component, Debug)]
pub struct TileSelected;
