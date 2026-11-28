use freyya_core::goods_market::{GoodsMarket, MarketGood, Observation, Unit, Value};
use freyya_core::{Rakaly, read_save};

const GRAIN: u16 = 7;
const CLOTHES: u16 = 12;
const IRON: u16 = 25;

fn manual_save_market() -> GoodsMarket {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/saves/sweden_1837_08_16.v3"
    );
    let save = std::fs::read(path).expect("the manual Save fixture is readable");
    read_save(&save, &Rakaly::on_path())
        .expect("the manual Save is read")
        .goods_market
        .expect("the manual Save names the player's market")
}

fn good(market: &GoodsMarket, good: u16) -> &MarketGood {
    market
        .goods
        .iter()
        .find(|entry| entry.good == good)
        .unwrap_or_else(|| panic!("good {good} is in the market"))
}

fn read_value(observation: &Observation) -> f64 {
    match &observation.value {
        Value::Read(value) => *value,
        Value::Missing { reason } => panic!("expected a read value, got missing: {reason}"),
    }
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-4,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn manual_save_gives_the_price_of_each_good_in_the_players_market() {
    let market = manual_save_market();
    let grain = good(&market, GRAIN);
    assert_close(read_value(&grain.price), 11.6228);
    assert_eq!(grain.price.unit, Unit::PoundsPerUnit);
    assert_eq!(
        grain.price.source,
        "country_manager.database.24.budget.current_price_report.goods.7.value"
    );
    assert_close(read_value(&good(&market, CLOTHES).price), 37.67137);
    assert_close(read_value(&good(&market, IRON).price), 40.0852);
    assert_eq!(market.goods.len(), 54);
}
