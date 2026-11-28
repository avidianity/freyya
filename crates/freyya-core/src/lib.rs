mod gamestate;
mod rakaly;
mod snapshot;

pub use rakaly::Rakaly;
pub use snapshot::{GameDate, PlayerCountry, Refusal, Snapshot};

pub fn read_save(save: &[u8], rakaly: &Rakaly) -> Result<Snapshot, Refusal> {
    let melted = rakaly.melt(save)?;
    let gamestate = gamestate::Gamestate::parse(&melted)?;
    gamestate.into_snapshot()
}
