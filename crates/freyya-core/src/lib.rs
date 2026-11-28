use std::time::{Duration, Instant};

pub mod buildings_jobs;
mod catalog;
mod construction;
mod envelope;
mod gamestate;
pub mod goods_market;
mod money;
mod observation;
mod rakaly;
mod save;
mod save_reader;
mod snapshot;

pub use catalog::{
    Area, AreaCheck, Catalog, CatalogRefusal, Define, Definition, Verdict, check_game_files,
    read_game_files,
};
pub use construction::{Construction, QueuedConstruction};
pub use money::Money;
pub use observation::{Observation, Reading, Unit};
pub use rakaly::Rakaly;
pub use save_reader::{SaveRead, SaveReader};
pub use snapshot::{Cause, GameDate, PlayerCountry, Refusal, Snapshot};

pub fn read_save(save: &[u8], rakaly: &Rakaly) -> Result<Snapshot, Refusal> {
    read_save_timed(save, rakaly).map(|(snapshot, _)| snapshot)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadTimes {
    pub check: Duration,
    pub decode: Duration,
    pub parse: Duration,
}

pub fn read_save_timed(save: &[u8], rakaly: &Rakaly) -> Result<(Snapshot, ReadTimes), Refusal> {
    let started = Instant::now();
    envelope::check_complete(save)?;
    let checked = Instant::now();
    let melted = rakaly.melt(save)?;
    let decoded = Instant::now();
    let snapshot = gamestate::into_snapshot(crate::save::SaveGamestate::parse(&melted)?)?;
    let parsed = Instant::now();
    let times = ReadTimes {
        check: checked - started,
        decode: decoded - checked,
        parse: parsed - decoded,
    };
    Ok((snapshot, times))
}
