mod construction;
mod envelope;
mod gamestate;
mod observation;
mod rakaly;
mod snapshot;

pub use construction::{Construction, QueuedConstruction};
pub use observation::{Observation, Unit};
pub use rakaly::Rakaly;
pub use snapshot::{GameDate, PlayerCountry, Refusal, Snapshot};

pub fn read_save(save: &[u8], rakaly: &Rakaly) -> Result<Snapshot, Refusal> {
    envelope::check_complete(save)?;
    let melted = rakaly.melt(save)?;
    let gamestate = gamestate::Gamestate::parse(&melted)?;
    gamestate.into_snapshot()
}
