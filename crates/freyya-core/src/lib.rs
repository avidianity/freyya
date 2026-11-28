mod catalog;
mod envelope;
mod gamestate;
mod money;
mod observation;
mod rakaly;
mod snapshot;

pub use catalog::{
    Area, AreaCheck, Catalog, CatalogRefusal, Define, Definition, Verdict, check_game_files,
    read_game_files,
};
pub use money::Money;
pub use observation::{Observation, Reading, Unit};
pub use rakaly::Rakaly;
pub use snapshot::{GameDate, PlayerCountry, Refusal, Snapshot};

pub fn read_save(save: &[u8], rakaly: &Rakaly) -> Result<Snapshot, Refusal> {
    envelope::check_complete(save)?;
    let melted = rakaly.melt(save)?;
    let gamestate = gamestate::Gamestate::parse(&melted)?;
    gamestate.into_snapshot()
}
