use colored::{Color, ColoredString, Colorize};
use std::{collections::BTreeMap, ops::Deref, str::FromStr};

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct Tags(BTreeMap<String, Style>);

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Style {
    pub fg: String,
    pub bg: Option<String>,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fg: "green".to_string(),
            bg: None,
        }
    }
}

impl Tags {
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Style)> {
        self.0.iter().map(|(t, s)| (t.as_str(), s))
    }

    pub fn set_fg(&mut self, tag: &str, fg: Color) {
        self.0.entry(tag.to_string()).or_default().fg = colour_string(fg);
    }

    pub fn set_bg(&mut self, tag: &str, bg: Color) {
        self.0.entry(tag.to_string()).or_default().bg = Some(colour_string(bg));
    }

    pub fn colourise(&self, tag: &str, text: &str) -> ColoredString {
        match self.0.get(tag) {
            Some(Style { fg, bg }) => {
                let mut s = text.color(fg.parse().unwrap_or(Color::White));
                if let Some(bg) = bg.as_ref().and_then(|x| x.parse::<Color>().ok()) {
                    s = s.on_color(bg);
                }
                s
            }
            None => text.into(),
        }
    }
}

fn colour_string(c: Color) -> String {
    match c {
        Color::Black => "black",
        Color::Red => "red",
        Color::Green => "green",
        Color::Yellow => "yellow",
        Color::Blue => "blue",
        Color::Magenta => "magenta",
        Color::Cyan => "cyan",
        Color::White => "white",
        Color::BrightBlack => "bright black",
        Color::BrightRed => "bright red",
        Color::BrightGreen => "bright green",
        Color::BrightYellow => "bright yellow",
        Color::BrightBlue => "bright blue",
        Color::BrightMagenta => "bright magenta",
        Color::BrightCyan => "bright cyan",
        Color::BrightWhite => "bright white",
        Color::TrueColor { .. } => "black",
    }
    .to_string()
}

#[derive(Clone)]
pub struct AddTag(String);

impl Deref for AddTag {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}
impl From<AddTag> for String {
    fn from(value: AddTag) -> Self {
        value.0
    }
}
impl FromStr for AddTag {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.strip_prefix('+')
            .map(|x| AddTag(x.to_string()))
            .ok_or("tag must start with +")
    }
}

#[derive(Clone)]
pub struct NegTag(String);

impl Deref for NegTag {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}
impl From<NegTag> for String {
    fn from(value: NegTag) -> Self {
        value.0
    }
}
impl FromStr for NegTag {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.strip_prefix('/')
            .map(|x| NegTag(x.to_string()))
            .ok_or("negation tag must start with /")
    }
}

#[derive(Clone)]
pub enum FilterTag {
    Add(AddTag),
    Neg(NegTag),
}

impl Deref for FilterTag {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Add(x) => x,
            Self::Neg(x) => x,
        }
    }
}
impl From<FilterTag> for String {
    fn from(value: FilterTag) -> Self {
        match value {
            FilterTag::Add(x) => String::from(x),
            FilterTag::Neg(x) => String::from(x),
        }
    }
}
impl FromStr for FilterTag {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        AddTag::from_str(s)
            .map(FilterTag::Add)
            .or_else(|_| NegTag::from_str(s).map(FilterTag::Neg))
            .map_err(|_| "filter tag must start with + or /")
    }
}

impl FilterTag {
    pub fn is_neg(&self) -> bool {
        matches!(self, Self::Neg(_))
    }

    pub fn filter<'a>(&self, mut tags: impl Iterator<Item = &'a str>) -> bool {
        match self {
            Self::Add(f) => tags.any(|t| t.eq(f.deref())),
            Self::Neg(f) => tags.all(|t| t.ne(f.deref())),
        }
    }
}
