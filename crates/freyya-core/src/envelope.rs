use jomini::envelope::{JominiFile, JominiFileKind, SaveHeaderKind};

use crate::Refusal;

pub(crate) fn check_complete(save: &[u8]) -> Result<(), Refusal> {
    let file = JominiFile::from_slice(save).map_err(|error| Refusal::Damaged {
        detail: format!("not a Victoria 3 Save: {error}"),
    })?;
    let zipped = match file.header().kind() {
        SaveHeaderKind::Text | SaveHeaderKind::Binary => false,
        SaveHeaderKind::UnifiedText
        | SaveHeaderKind::UnifiedBinary
        | SaveHeaderKind::SplitText
        | SaveHeaderKind::SplitBinary => true,
        SaveHeaderKind::Other(kind) => {
            return Err(Refusal::Damaged {
                detail: format!("unknown Save kind {kind}"),
            });
        }
    };
    match file.kind() {
        JominiFileKind::Uncompressed(_) if zipped => Err(Refusal::Incomplete),
        JominiFileKind::Uncompressed(_) => Ok(()),
        JominiFileKind::Zip(zip) => {
            let mut gamestate = zip.gamestate_verified().map_err(|error| Refusal::Damaged {
                detail: format!("the gamestate cannot be found: {error}"),
            })?;
            std::io::copy(&mut gamestate, &mut std::io::sink())
                .map(|_| ())
                .map_err(|error| Refusal::Damaged {
                    detail: format!("the gamestate does not unpack intact: {error}"),
                })
        }
    }
}
