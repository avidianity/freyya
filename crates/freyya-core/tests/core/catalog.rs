use std::path::PathBuf;

use freyya_core::{CatalogRefusal, check_game_files, read_game_files};

use crate::helpers::empty_temp_folder;

const TINY_GAME: &[(&str, &str)] = &[
    (
        "launcher/launcher-settings.json",
        r#"{ "gameId": "victoria3", "version": "9.8.7 (Teapot)", "rawVersion": "9.8.7" }"#,
    ),
    (
        "game/common/goods/00_goods.txt",
        "\u{feff}# made-up goods\nwidgets = {\n\tcost = 11\n\tcategory = staple\n}\ngizmos = {\n\tcost = 22\n}\n",
    ),
    (
        "game/common/goods/01_more_goods.txt",
        "doohickeys = { cost = 33 }\n",
    ),
    (
        "game/common/buildings/00_buildings.txt",
        "building_widget_works = {\n\tbuilding_group = bg_test\n\tproduction_method_groups = { pmg_widgets }\n\tai_value = { value = 1 if = { limit = { owner ?= { always = yes } } add = 2 } }\n}\n",
    ),
    (
        "game/common/production_methods/00_pms.txt",
        "pm_hand_widgets = { building_modifiers = { workforce_scaled = { goods_output_widgets_add = 5 } } }\n",
    ),
    (
        "game/common/technology/technologies/00_tech.txt",
        "widgetry = { era = era_1 }\n",
    ),
    (
        "game/common/laws/00_laws.txt",
        "law_widget_tax = { group = lawgroup_test }\n",
    ),
    (
        "game/common/pop_needs/00_needs.txt",
        "popneed_widgets = { default = widgets entry = { goods = widgets weight = 1 } }\n",
    ),
    (
        "game/common/defines/00_defines.txt",
        "@made_up_rate = 0.5\nNGame {\n\tSTART_DATE = \"1900.1.1\"\n}\nNCountry {\n\tSOME_LIMIT = 4 # made up\n\tSOME_RATE = @made_up_rate\n}\n",
    ),
    (
        "game/map_data/state_regions/00_regions.txt",
        "STATE_TESTMARK = { id = 1 arable_land = 3 }\n",
    ),
    (
        "game/localization/english/test_l_english.yml",
        "\u{feff}l_english:\n widgets:0 \"Widgets\"\n gizmos:0 \"Gizmos\"\n doohickeys: \"Doohickeys\"\n building_widget_works:0 \"Widget Works\"\n pm_hand_widgets:0 \"Hand Widgets\"\n widgetry:0 \"Widgetry\"\n law_widget_tax:0 \"Widget Tax\"\n popneed_widgets:0 \"Widget Need\"\n STATE_TESTMARK:0 \"Testmark\"\n",
    ),
];

fn game_folder(test: &str, changes: &[(&str, Option<&str>)]) -> PathBuf {
    let root = empty_temp_folder(&format!("catalog-{test}"));
    let mut files: Vec<(&str, Option<&str>)> = TINY_GAME
        .iter()
        .map(|(path, text)| (*path, Some(*text)))
        .collect();
    for (path, text) in changes {
        match files.iter_mut().find(|(existing, _)| existing == path) {
            Some(file) => file.1 = *text,
            None => files.push((path, *text)),
        }
    }
    for (path, text) in files {
        if let Some(text) = text {
            let file = root.join(path);
            std::fs::create_dir_all(file.parent().expect("a file has a folder"))
                .expect("the test folder is writable");
            std::fs::write(file, text).expect("the test file is writable");
        }
    }
    root
}

#[test]
fn catalog_records_the_installed_game_version() {
    let folder = game_folder("version", &[]);

    let catalog = read_game_files(&folder).expect("the tiny game folder is read");

    assert_eq!(catalog.game_version, "9.8.7");
}

#[test]
fn goods_are_named_by_their_index_in_the_save() {
    let folder = game_folder("goods", &[]);

    let catalog = read_game_files(&folder).expect("the tiny game folder is read");

    let names: Vec<_> = (0..4).map(|index| catalog.good_name(index)).collect();
    assert_eq!(
        names,
        [Some("Widgets"), Some("Gizmos"), Some("Doohickeys"), None]
    );
}

#[test]
fn buildings_are_named_by_their_key_in_the_save() {
    let folder = game_folder("buildings", &[]);

    let catalog = read_game_files(&folder).expect("the tiny game folder is read");

    assert_eq!(
        catalog.building_name("building_widget_works"),
        Some("Widget Works")
    );
    assert_eq!(catalog.building_name("building_unknown"), None);
}

#[test]
fn rule_areas_list_their_definitions_with_english_names() {
    let folder = game_folder("areas", &[]);

    let catalog = read_game_files(&folder).expect("the tiny game folder is read");

    let named = |definitions: &[freyya_core::Definition]| -> Vec<(String, Option<String>)> {
        definitions
            .iter()
            .map(|definition| (definition.key.clone(), definition.name.clone()))
            .collect()
    };
    let pair = |key: &str, name: &str| vec![(key.to_owned(), Some(name.to_owned()))];
    assert_eq!(
        named(&catalog.production_methods),
        pair("pm_hand_widgets", "Hand Widgets")
    );
    assert_eq!(named(&catalog.technologies), pair("widgetry", "Widgetry"));
    assert_eq!(named(&catalog.laws), pair("law_widget_tax", "Widget Tax"));
    assert_eq!(
        named(&catalog.pop_needs),
        pair("popneed_widgets", "Widget Need")
    );
    assert_eq!(
        named(&catalog.state_regions),
        pair("STATE_TESTMARK", "Testmark")
    );
}

#[test]
fn defines_are_read_by_namespace_and_key() {
    let folder = game_folder("defines", &[]);

    let catalog = read_game_files(&folder).expect("the tiny game folder is read");

    let defines: Vec<(&str, &str, &str)> = catalog
        .defines
        .iter()
        .map(|define| {
            (
                define.namespace.as_str(),
                define.key.as_str(),
                define.value.as_str(),
            )
        })
        .collect();
    assert_eq!(
        defines,
        [
            ("NGame", "START_DATE", "1900.1.1"),
            ("NCountry", "SOME_LIMIT", "4"),
            ("NCountry", "SOME_RATE", "0.5"),
        ]
    );
}

const BROKEN_GOODS: (&str, Option<&str>) = (
    "game/common/goods/01_more_goods.txt",
    Some("doohickeys = }\n"),
);

#[test]
fn catalog_refuses_a_game_file_it_cannot_parse_and_names_it() {
    let folder = game_folder("broken", &[BROKEN_GOODS]);

    let refusal = read_game_files(&folder).expect_err("a broken goods file is refused");

    let CatalogRefusal::Unreadable { path, .. } = refusal;
    assert!(path.ends_with("01_more_goods.txt"), "{}", path.display());
}

#[test]
fn catalog_refusal_keeps_the_original_error_as_its_cause() {
    let folder = game_folder("broken-cause", &[BROKEN_GOODS]);

    let refusal = read_game_files(&folder).expect_err("a broken goods file is refused");

    assert!(std::error::Error::source(&refusal).is_some(), "{refusal}");
}

fn verdicts(folder: &std::path::Path) -> Vec<(String, bool)> {
    check_game_files(folder)
        .into_iter()
        .map(|check| (check.area.to_string(), check.verdict.passed()))
        .collect()
}

fn all_pass_except(failing: &[&str]) -> Vec<(String, bool)> {
    [
        "version",
        "English names",
        "goods",
        "buildings",
        "production methods",
        "technology",
        "laws",
        "pop needs",
        "defines",
        "state regions",
    ]
    .into_iter()
    .map(|area| (area.to_owned(), !failing.contains(&area)))
    .collect()
}

#[test]
fn check_passes_every_area_of_a_complete_game_folder() {
    let folder = game_folder("check-pass", &[]);

    assert_eq!(verdicts(&folder), all_pass_except(&[]));
}

#[test]
fn check_fails_only_the_area_with_a_broken_file() {
    let folder = game_folder("check-broken", &[BROKEN_GOODS]);

    assert_eq!(verdicts(&folder), all_pass_except(&["goods"]));
}

#[test]
fn check_fails_an_area_whose_definitions_have_no_english_name() {
    let folder = game_folder(
        "check-unnamed",
        &[(
            "game/common/buildings/01_more.txt",
            Some("building_nameless_works = { building_group = bg_test }\n"),
        )],
    );

    let checks = check_game_files(&folder);

    let buildings = checks
        .iter()
        .find(|check| check.area.to_string() == "buildings")
        .expect("the check covers buildings");
    assert!(!buildings.verdict.passed());
    assert!(
        buildings
            .verdict
            .to_string()
            .contains("building_nameless_works"),
        "{}",
        buildings.verdict
    );
}

#[test]
fn check_fails_every_area_when_the_folder_is_not_a_game_folder() {
    let folder = game_folder("check-empty", &[]).join("not-there");

    assert_eq!(
        verdicts(&folder),
        all_pass_except(&[
            "version",
            "English names",
            "goods",
            "buildings",
            "production methods",
            "technology",
            "laws",
            "pop needs",
            "defines",
            "state regions",
        ])
    );
}
