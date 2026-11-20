use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const CONFIG_PATH: &str = "rustro_config.toml";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Field {
    PineTree,
    Pumpkin,
    Pepper,
    Sunflower,
    Dandelion,
    Clover,
    BlueFlower,
    Strawberry,
}

impl Field {
    pub const ALL: &'static [Field] = &[
        Field::PineTree,
        Field::Pumpkin,
        Field::Pepper,
        Field::Sunflower,
        Field::Dandelion,
        Field::Clover,
        Field::BlueFlower,
        Field::Strawberry,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Field::PineTree => "Pine Tree Forest",
            Field::Pumpkin => "Pumpkin Patch",
            Field::Pepper => "Pepper Patch",
            Field::Sunflower => "Sunflower Field",
            Field::Dandelion => "Dandelion Field",
            Field::Clover => "Clover Field",
            Field::BlueFlower => "Blue Flower Field",
            Field::Strawberry => "Strawberry Field",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternKind {
    Squares,
    Snake,
    CornerXSnake,
    Stationary,
}

impl PatternKind {
    pub const ALL: &'static [PatternKind] = &[
        PatternKind::Squares,
        PatternKind::Snake,
        PatternKind::CornerXSnake,
        PatternKind::Stationary,
    ];

    pub fn label(self) -> &'static str {
        match self {
            PatternKind::Squares => "Squares",
            PatternKind::Snake => "Snake",
            PatternKind::CornerXSnake => "CornerXSnake",
            PatternKind::Stationary => "Stationary",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroConfig {
    pub field: Field,
    pub pattern: PatternKind,
    pub hive_slot: u8,
    pub gather_minutes: u32,
    pub move_speed: f32,
    pub pattern_size: u8,
    pub convert_at_full: bool,
    pub auto_click_gather: bool,
}

impl Default for MacroConfig {
    fn default() -> Self {
        Self {
            field: Field::PineTree,
            pattern: PatternKind::CornerXSnake,
            hive_slot: 1,
            gather_minutes: 10,
            move_speed: 28.0,
            pattern_size: 3,
            convert_at_full: true,
            auto_click_gather: true,
        }
    }
}

impl MacroConfig {
    pub fn load() -> Self {
        let path = Path::new(CONFIG_PATH);
        if let Ok(raw) = fs::read_to_string(path) {
            if let Ok(cfg) = toml::from_str::<MacroConfig>(&raw) {
                return cfg;
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), String> {
        let raw = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(CONFIG_PATH, raw).map_err(|e| e.to_string())
    }
}
