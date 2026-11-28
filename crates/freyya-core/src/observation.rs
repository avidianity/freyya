use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Observation {
    pub reading: Reading,
    pub unit: Unit,
    pub source: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Reading {
    Read(f64),
    Missing,
}

pub(crate) fn observe(value: Option<f64>, unit: Unit, source: String) -> Observation {
    Observation {
        reading: value.map_or(Reading::Missing, Reading::Read),
        unit,
        source,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Pounds,
    PoundsPerWeek,
    ConstructionPoints,
    ConstructionPointsPerWeek,
}

impl fmt::Display for Observation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.reading {
            Reading::Read(value) => write!(f, "{value:.2} {}", self.unit)?,
            Reading::Missing => f.write_str("missing")?,
        }
        write!(f, "  [{}]", self.source)
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unit::Pounds => f.write_str("£"),
            Unit::PoundsPerWeek => f.write_str("£/week"),
            Unit::ConstructionPoints => f.write_str("construction points"),
            Unit::ConstructionPointsPerWeek => f.write_str("construction points/week"),
        }
    }
}
