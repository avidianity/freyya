mod buildings_jobs;
mod config;
mod measure;

use std::path::Path;
use std::process::ExitCode;

use freyya_core::GoodsMarket;
use freyya_core::{
    Catalog, Construction, QueuedConstruction, Rakaly, check_game_files, read_game_files, read_save,
};

use config::{Command, ConfigError, command_from_args};

fn main() -> ExitCode {
    match command_from_args(std::env::args().skip(1)) {
        Ok(Command::ReadSave { save, rakaly, game }) => print_save(&save, &rakaly, game.as_deref()),
        Ok(Command::CheckGameFiles { game }) => print_game_files_check(&game),
        Ok(Command::MeasureSave { saves, rakaly }) => measure::print_measurements(&saves, &rakaly),
        Err(problem) => {
            eprintln!("{problem}");
            match problem {
                ConfigError::Usage(_) => ExitCode::from(64),
                ConfigError::MissingPath(_) => ExitCode::from(66),
            }
        }
    }
}

fn print_save(path: &Path, rakaly: &Rakaly, game: Option<&Path>) -> ExitCode {
    let catalog = match game.map(read_game_files).transpose() {
        Ok(catalog) => catalog,
        Err(refusal) => {
            eprintln!("Refused the Game files: {refusal}");
            return ExitCode::from(66);
        }
    };
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("cannot read {}: {error}", path.display());
            return ExitCode::from(66);
        }
    };
    match read_save(&bytes, rakaly) {
        Ok(snapshot) => {
            println!("Save:           {}", path.display());
            println!("Game version:   {}", snapshot.game_version);
            println!("Game date:      {}", snapshot.game_date);
            println!(
                "Player country: {} ({})",
                snapshot.player_country.name, snapshot.player_country.tag
            );
            if let Some(catalog) = &catalog {
                print_catalog_version(catalog, &snapshot.game_version);
            }
            println!("Treasury:       {}", snapshot.money.treasury);
            println!("Income:         {}", snapshot.money.weekly_income);
            println!("Expenses:       {}", snapshot.money.weekly_expenses);
            println!("Debt:           {}", snapshot.money.debt);
            buildings_jobs::print_buildings_and_jobs(
                &snapshot.buildings_and_jobs,
                catalog.as_ref(),
            );
            print_construction(&snapshot.construction, catalog.as_ref());
            print_goods_market(snapshot.goods_market.as_ref(), catalog.as_ref());
            ExitCode::SUCCESS
        }
        Err(refusal) => {
            eprintln!("Refused {}: {refusal}", path.display());
            ExitCode::from(65)
        }
    }
}

fn print_construction(construction: &Construction, catalog: Option<&Catalog>) {
    println!("Construction in use: {}", construction.construction_in_use);
    print_queue("Government queue", &construction.government_queue, catalog);
    print_queue("Private queue", &construction.private_queue, catalog);
}

fn print_queue(title: &str, queue: &[QueuedConstruction], catalog: Option<&Catalog>) {
    println!("{title}: {} items", queue.len());
    for (position, item) in queue.iter().enumerate() {
        println!(
            "  {:>2}. {} in state {}",
            position + 1,
            building_label(&item.building, catalog),
            item.state
        );
        println!("      left:  {}", item.construction_left);
        println!("      speed: {}", item.construction_speed);
    }
}

fn print_catalog_version(catalog: &Catalog, save_version: &str) {
    println!("Game files:     version {}", catalog.game_version);
    if catalog.game_version != save_version {
        println!(
            "Warning:        the Game files ({}) and the Save ({save_version}) differ in version, so names may be wrong",
            catalog.game_version
        );
    }
}

fn print_game_files_check(game: &Path) -> ExitCode {
    println!("Game files: {}", game.display());
    let checks = check_game_files(game);
    for check in &checks {
        println!("{:<20}{}", check.area.to_string(), check.verdict);
    }
    if !checks.iter().all(|check| check.verdict.passed()) {
        return ExitCode::FAILURE;
    }
    match read_game_files(game) {
        Ok(catalog) => {
            print_catalog(&catalog);
            ExitCode::SUCCESS
        }
        Err(refusal) => {
            eprintln!("Refused the Game files: {refusal}");
            ExitCode::FAILURE
        }
    }
}

fn print_catalog(catalog: &Catalog) {
    println!("Catalog:    version {}", catalog.game_version);
    for (area, count) in [
        ("goods", catalog.goods.len()),
        ("buildings", catalog.buildings.len()),
        ("production methods", catalog.production_methods.len()),
        ("technologies", catalog.technologies.len()),
        ("laws", catalog.laws.len()),
        ("pop needs", catalog.pop_needs.len()),
        ("state regions", catalog.state_regions.len()),
        ("defines", catalog.defines.len()),
    ] {
        println!("  {area:<20}{count}");
    }
}

pub(crate) fn building_label(key: &str, catalog: Option<&Catalog>) -> String {
    match catalog.and_then(|catalog| catalog.building_name(key)) {
        Some(name) if name != key => format!("{name} ({key})"),
        _ => key.to_owned(),
    }
}

fn print_goods_market(market: Option<&GoodsMarket>, catalog: Option<&Catalog>) {
    let Some(market) = market else {
        println!("Goods market:   missing (the Save names no market for the player's country)");
        return;
    };
    println!(
        "Goods market:   {} (owner {}; members {})",
        market.market,
        market.owner.as_deref().unwrap_or("missing"),
        market.members.join(", ")
    );
    for good in &market.goods {
        println!("  {}", good_label(good.good, catalog));
        println!("    Price:                      {}", good.price);
        println!(
            "    Sell orders from buildings: {}",
            good.sell_orders_from_buildings
        );
        println!(
            "    Buy orders from buildings:  {}",
            good.buy_orders_from_buildings
        );
        println!(
            "    Buy orders from pops:       {}",
            good.buy_orders_from_pops
        );
        println!("    Net exports:                {}", good.net_exports);
    }
}

fn good_label(index: u16, catalog: Option<&Catalog>) -> String {
    match catalog.and_then(|catalog| catalog.good_name(usize::from(index))) {
        Some(name) => format!("{name} (good {index})"),
        None => format!("Good {index}"),
    }
}
