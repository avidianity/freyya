use std::fmt;

use serde::Serialize;

use crate::buildings_jobs::BuildingsAndJobs;
use crate::goods_market::GoodsMarket;
use crate::{Construction, Money};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Snapshot {
    pub game_version: String,
    pub game_date: GameDate,
    pub player_country: PlayerCountry,
    pub money: Money,
    pub buildings_and_jobs: BuildingsAndJobs,
    pub construction: Construction,
    pub goods_market: Option<GoodsMarket>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlayerCountry {
    pub tag: String,
    pub name: String,
}

pub type Cause = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug)]
pub enum Refusal {
    Damaged {
        detail: String,
        cause: Option<Cause>,
    },
    Incomplete,
    UnsupportedVersion {
        version: String,
    },
    Modded {
        mods: Vec<String>,
    },
    ReaderUnavailable {
        detail: String,
        cause: Option<Cause>,
    },
}

impl Refusal {
    pub(crate) fn damaged(detail: impl Into<String>) -> Self {
        Refusal::Damaged {
            detail: detail.into(),
            cause: None,
        }
    }

    pub(crate) fn damaged_by(detail: impl Into<String>, cause: impl Into<Cause>) -> Self {
        Refusal::Damaged {
            detail: detail.into(),
            cause: Some(cause.into()),
        }
    }

    pub(crate) fn reader_unavailable(detail: impl Into<String>) -> Self {
        Refusal::ReaderUnavailable {
            detail: detail.into(),
            cause: None,
        }
    }

    pub(crate) fn reader_unavailable_by(
        detail: impl Into<String>,
        cause: impl Into<Cause>,
    ) -> Self {
        Refusal::ReaderUnavailable {
            detail: detail.into(),
            cause: Some(cause.into()),
        }
    }
}

fn write_detail(f: &mut fmt::Formatter<'_>, detail: &str, cause: Option<&Cause>) -> fmt::Result {
    f.write_str(detail)?;
    match cause {
        Some(cause) => write!(f, ": {cause}"),
        None => Ok(()),
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refusal::Damaged { detail, cause } => {
                f.write_str("the Save is damaged: ")?;
                write_detail(f, detail, cause.as_ref())
            }
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
            Refusal::ReaderUnavailable { detail, cause } => {
                f.write_str("the Save reader (Rakaly CLI) could not run: ")?;
                write_detail(f, detail, cause.as_ref())
            }
        }
    }
}

impl std::error::Error for Refusal {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Refusal::Damaged { cause, .. } | Refusal::ReaderUnavailable { cause, .. } => {
                cause.as_deref().map(|cause| cause as _)
            }
            Refusal::Incomplete | Refusal::UnsupportedVersion { .. } | Refusal::Modded { .. } => {
                None
            }
        }
    }
}
