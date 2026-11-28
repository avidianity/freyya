use std::fmt;

use crate::{Construction, Money};

#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    pub game_version: String,
    pub game_date: GameDate,
    pub player_country: PlayerCountry,
    pub money: Money,
    pub construction: Construction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameDate {
    pub year: i16,
    pub month: u8,
    pub day: u8,
}

impl fmt::Display for GameDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerCountry {
    pub tag: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    Damaged { detail: String },
    Incomplete,
    UnsupportedVersion { version: String },
    Modded { mods: Vec<String> },
    ReaderUnavailable { detail: String },
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refusal::Damaged { detail } => write!(f, "the Save is damaged: {detail}"),
            Refusal::Incomplete => write!(
                f,
                "the Save is incomplete: its end is missing, so it may still be being written"
            ),
            Refusal::UnsupportedVersion { version } => write!(
                f,
                "the Save comes from game version {version}, which freyya has not checked"
            ),
            Refusal::Modded { mods } => write!(
                f,
                "the Save has mods active ({}); freyya supports only vanilla Saves",
                mods.join(", ")
            ),
            Refusal::ReaderUnavailable { detail } => {
                write!(f, "the Save reader (Rakaly CLI) could not run: {detail}")
            }
        }
    }
}

impl std::error::Error for Refusal {}
