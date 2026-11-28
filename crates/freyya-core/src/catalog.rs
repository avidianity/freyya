use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

use jomini::text::ValueReader;
use jomini::{DeserializeError, Encoding, TextTape};
use serde::Deserialize;

use crate::Cause;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Catalog {
    pub game_version: String,
    pub goods: Vec<Definition>,
    pub buildings: Vec<Definition>,
    pub production_methods: Vec<Definition>,
    pub technologies: Vec<Definition>,
    pub laws: Vec<Definition>,
    pub pop_needs: Vec<Definition>,
    pub state_regions: Vec<Definition>,
    pub defines: Vec<Define>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Define {
    pub namespace: String,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Definition {
    pub key: String,
    pub name: Option<String>,
}

impl Definition {
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.key)
    }
}

impl Catalog {
    pub fn good_name(&self, save_index: usize) -> Option<&str> {
        self.goods.get(save_index).map(Definition::display_name)
    }

    pub fn building_name(&self, key: &str) -> Option<&str> {
        self.buildings
            .iter()
            .find(|building| building.key == key)
            .map(Definition::display_name)
    }
}

#[derive(Debug)]
pub enum CatalogRefusal {
    Unreadable {
        path: PathBuf,
        detail: Option<String>,
        cause: Cause,
    },
}

impl CatalogRefusal {
    fn unreadable(path: &Path, cause: impl Into<Cause>) -> Self {
        CatalogRefusal::Unreadable {
            path: path.to_path_buf(),
            detail: None,
            cause: cause.into(),
        }
    }

    fn unreadable_because(path: &Path, detail: String, cause: impl Into<Cause>) -> Self {
        CatalogRefusal::Unreadable {
            path: path.to_path_buf(),
            detail: Some(detail),
            cause: cause.into(),
        }
    }
}

impl fmt::Display for CatalogRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CatalogRefusal::Unreadable {
                path,
                detail,
                cause,
            } => {
                write!(f, "cannot read {}: ", path.display())?;
                if let Some(detail) = detail {
                    write!(f, "{detail}: ")?;
                }
                write!(f, "{cause}")
            }
        }
    }
}

impl std::error::Error for CatalogRefusal {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CatalogRefusal::Unreadable { cause, .. } => Some(cause.as_ref()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Area {
    Version,
    EnglishNames,
    Goods,
    Buildings,
    ProductionMethods,
    Technology,
    Laws,
    PopNeeds,
    Defines,
    StateRegions,
}

impl Area {
    const ALL: [Area; 10] = [
        Area::Version,
        Area::EnglishNames,
        Area::Goods,
        Area::Buildings,
        Area::ProductionMethods,
        Area::Technology,
        Area::Laws,
        Area::PopNeeds,
        Area::Defines,
        Area::StateRegions,
    ];

    fn path(self, game_folder: &Path) -> PathBuf {
        let game = game_folder.join("game");
        let common = game.join("common");
        match self {
            Area::Version => game_folder.join("launcher").join("launcher-settings.json"),
            Area::EnglishNames => game.join("localization").join("english"),
            Area::Goods => common.join("goods"),
            Area::Buildings => common.join("buildings"),
            Area::ProductionMethods => common.join("production_methods"),
            Area::Technology => common.join("technology").join("technologies"),
            Area::Laws => common.join("laws"),
            Area::PopNeeds => common.join("pop_needs"),
            Area::Defines => common.join("defines"),
            Area::StateRegions => game.join("map_data").join("state_regions"),
        }
    }
}

impl fmt::Display for Area {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Area::Version => "version",
            Area::EnglishNames => "English names",
            Area::Goods => "goods",
            Area::Buildings => "buildings",
            Area::ProductionMethods => "production methods",
            Area::Technology => "technology",
            Area::Laws => "laws",
            Area::PopNeeds => "pop needs",
            Area::Defines => "defines",
            Area::StateRegions => "state regions",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AreaCheck {
    pub area: Area,
    pub verdict: Verdict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Pass { summary: String },
    Fail { problem: String },
}

impl Verdict {
    pub fn passed(&self) -> bool {
        matches!(self, Verdict::Pass { .. })
    }
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Verdict::Pass { summary } => write!(f, "pass  {summary}"),
            Verdict::Fail { problem } => write!(f, "FAIL  {problem}"),
        }
    }
}

const UNNAMED_SHOWN: usize = 5;

pub fn read_game_files(game_folder: &Path) -> Result<Catalog, CatalogRefusal> {
    let game_version = read_game_version(&Area::Version.path(game_folder))?;
    let names = read_english_names(&Area::EnglishNames.path(game_folder))?;
    let definitions = |area: Area| read_definitions(&area.path(game_folder), &names);
    Ok(Catalog {
        game_version,
        goods: definitions(Area::Goods)?,
        buildings: definitions(Area::Buildings)?,
        production_methods: definitions(Area::ProductionMethods)?,
        technologies: definitions(Area::Technology)?,
        laws: definitions(Area::Laws)?,
        pop_needs: definitions(Area::PopNeeds)?,
        state_regions: definitions(Area::StateRegions)?,
        defines: read_defines(&Area::Defines.path(game_folder))?,
    })
}

pub fn check_game_files(game_folder: &Path) -> Vec<AreaCheck> {
    let names = read_english_names(&Area::EnglishNames.path(game_folder));
    Area::ALL
        .into_iter()
        .map(|area| {
            let path = area.path(game_folder);
            let verdict = match area {
                Area::Version => verdict(read_game_version(&path), Ok),
                Area::EnglishNames => verdict(names.as_ref(), |names| {
                    if names.is_empty() {
                        Err("no English names found".to_owned())
                    } else {
                        Ok(format!("{} names", names.len()))
                    }
                }),
                Area::Defines => verdict(read_defines(&path), |defines| {
                    if defines.is_empty() {
                        Err("no defines found".to_owned())
                    } else {
                        Ok(format!("{} defines", defines.len()))
                    }
                }),
                _ => {
                    let no_names = HashMap::new();
                    let names = names.as_ref().unwrap_or(&no_names);
                    verdict(read_definitions(&path, names), definitions_verdict)
                }
            };
            AreaCheck { area, verdict }
        })
        .collect()
}

fn verdict<T, E: fmt::Display>(
    read: Result<T, E>,
    judge: impl FnOnce(T) -> Result<String, String>,
) -> Verdict {
    match read.map_err(|refusal| refusal.to_string()).and_then(judge) {
        Ok(summary) => Verdict::Pass { summary },
        Err(problem) => Verdict::Fail { problem },
    }
}

fn definitions_verdict(definitions: Vec<Definition>) -> Result<String, String> {
    let unnamed: Vec<&str> = definitions
        .iter()
        .filter(|definition| definition.name.is_none())
        .map(|definition| definition.key.as_str())
        .collect();
    match (definitions.len(), unnamed.len()) {
        (0, _) => Err("no definitions found".to_owned()),
        (total, 0) => Ok(format!("{total} definitions, all named")),
        (total, missing) => Err(format!(
            "{missing} of {total} definitions have no English name, such as {}",
            unnamed[..missing.min(UNNAMED_SHOWN)].join(", ")
        )),
    }
}

fn read_defines(folder: &Path) -> Result<Vec<Define>, CatalogRefusal> {
    let mut defines = Vec::new();
    for path in files_with_extension(folder, "txt")? {
        let text = read_file(&path)?;
        let tape = parse_script(&path, &text)?;
        let reader = tape.utf8_reader();
        let (variables, namespaces): (Vec<_>, Vec<_>) = reader
            .fields()
            .partition(|(key, _, _)| key.read_str().starts_with('@'));
        let variables = variables
            .into_iter()
            .map(|(key, _, value)| Ok((key.read_string(), define_value(&value)?)))
            .collect::<Result<HashMap<_, _>, DeserializeError>>()
            .map_err(|error| {
                CatalogRefusal::unreadable_because(
                    &path,
                    "a script variable has no plain value".to_owned(),
                    error,
                )
            })?;
        for (namespace, _, block) in namespaces {
            let namespace = namespace.read_string();
            let block = block.read_object().map_err(|error| {
                CatalogRefusal::unreadable_because(
                    &path,
                    format!("define namespace {namespace} is not a block"),
                    error,
                )
            })?;
            for (key, _, value) in block.fields() {
                let key = key.read_string();
                if key.starts_with('@') {
                    continue;
                }
                let value = define_value(&value).map_err(|error| {
                    CatalogRefusal::unreadable_because(
                        &path,
                        format!("define {namespace}.{key} has no plain value"),
                        error,
                    )
                })?;
                let value = variables.get(&value).cloned().unwrap_or(value);
                defines.push(Define {
                    namespace: namespace.clone(),
                    key,
                    value,
                });
            }
        }
    }
    Ok(defines)
}

fn define_value<E: Encoding + Clone>(
    value: &ValueReader<'_, '_, E>,
) -> Result<String, DeserializeError> {
    if let Ok(scalar) = value.read_string() {
        return Ok(scalar);
    }
    let items = value
        .read_array()?
        .values()
        .map(|item| item.read_string())
        .collect::<Result<Vec<_>, _>>()?;
    Ok(format!("{{ {} }}", items.join(" ")))
}

fn parse_script<'a>(path: &Path, text: &'a [u8]) -> Result<TextTape<'a>, CatalogRefusal> {
    TextTape::from_slice(text).map_err(|error| CatalogRefusal::unreadable(path, error))
}

#[derive(Deserialize)]
struct LauncherSettings {
    #[serde(rename = "rawVersion")]
    raw_version: String,
}

fn read_game_version(path: &Path) -> Result<String, CatalogRefusal> {
    let text = read_file(path)?;
    let settings: LauncherSettings = serde_json::from_slice(&text).map_err(|error| {
        CatalogRefusal::unreadable_because(path, "no game version".to_owned(), error)
    })?;
    Ok(settings.raw_version)
}

fn read_definitions(
    folder: &Path,
    names: &HashMap<String, String>,
) -> Result<Vec<Definition>, CatalogRefusal> {
    let mut definitions = Vec::new();
    for path in files_with_extension(folder, "txt")? {
        let text = read_file(&path)?;
        let tape = parse_script(&path, &text)?;
        for (key, _, _) in tape.utf8_reader().fields() {
            let key = key.read_string();
            if key.starts_with('@') {
                continue;
            }
            let name = names.get(&key).cloned();
            definitions.push(Definition { key, name });
        }
    }
    Ok(definitions)
}

fn read_english_names(folder: &Path) -> Result<HashMap<String, String>, CatalogRefusal> {
    let mut names = HashMap::new();
    for path in files_with_extension(folder, "yml")? {
        let bytes = read_file(&path)?;
        let text =
            String::from_utf8(bytes).map_err(|error| CatalogRefusal::unreadable(&path, error))?;
        names.extend(text.lines().filter_map(localization_entry));
    }
    Ok(names)
}

fn localization_entry(line: &str) -> Option<(String, String)> {
    let line = line.trim_start();
    if line.starts_with('#') {
        return None;
    }
    let (key, rest) = line.split_once(':')?;
    let open = rest.find('"')?;
    let close = rest.rfind('"')?;
    if close <= open || key.contains(char::is_whitespace) {
        return None;
    }
    Some((key.to_owned(), rest[open + 1..close].to_owned()))
}

fn files_with_extension(folder: &Path, extension: &str) -> Result<Vec<PathBuf>, CatalogRefusal> {
    let mut files = Vec::new();
    let mut pending = vec![folder.to_path_buf()];
    while let Some(dir) = pending.pop() {
        let entries =
            std::fs::read_dir(&dir).map_err(|error| CatalogRefusal::unreadable(&dir, error))?;
        for entry in entries {
            let path = entry
                .map_err(|error| CatalogRefusal::unreadable(&dir, error))?
                .path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|ext| ext == extension) {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn read_file(path: &Path) -> Result<Vec<u8>, CatalogRefusal> {
    std::fs::read(path).map_err(|error| CatalogRefusal::unreadable(path, error))
}
