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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Pounds,
    PoundsPerWeek,
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
        }
    }
}
