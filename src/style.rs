use std::fmt;
use std::str::FromStr;

/// SQL formatting style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Style {
    /// Simon Holywell's river style — keywords right-aligned to form a visual river.
    #[default]
    River,
    /// Mozilla style — keywords left-aligned, content indented 4 spaces.
    Mozilla,
    /// AWeber style — river style with JOINs participating in keyword alignment.
    Aweber,
    /// dbt style — Mozilla-like with lowercase keywords and blank lines between clauses.
    Dbt,
    /// GitLab style — Mozilla-like with 2-space indent and uppercase keywords.
    Gitlab,
    /// Kickstarter style — Mozilla-like with 2-space indent and compact JOINs.
    Kickstarter,
    /// mattmc3 style — lowercase river with leading commas.
    Mattmc3,
}

impl Style {
    /// All available styles.
    pub const ALL: &[Style] = &[
        Style::River,
        Style::Mozilla,
        Style::Aweber,
        Style::Dbt,
        Style::Gitlab,
        Style::Kickstarter,
        Style::Mattmc3,
    ];
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Style::River => write!(f, "river"),
            Style::Mozilla => write!(f, "mozilla"),
            Style::Aweber => write!(f, "aweber"),
            Style::Dbt => write!(f, "dbt"),
            Style::Gitlab => write!(f, "gitlab"),
            Style::Kickstarter => write!(f, "kickstarter"),
            Style::Mattmc3 => write!(f, "mattmc3"),
        }
    }
}

impl FromStr for Style {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "river" => Ok(Style::River),
            "mozilla" => Ok(Style::Mozilla),
            "aweber" => Ok(Style::Aweber),
            "dbt" => Ok(Style::Dbt),
            "gitlab" => Ok(Style::Gitlab),
            "kickstarter" => Ok(Style::Kickstarter),
            "mattmc3" => Ok(Style::Mattmc3),
            _ => Err(format!("Unsupported style: '{s}'")),
        }
    }
}
