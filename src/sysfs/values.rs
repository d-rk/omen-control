//! Typed sysfs values and their text formats.
//!
//! Read formats follow what the driver emits (`#rrggbb`, `(unset)`, `(cpu)`, `n/a`, ...); write formats follow what
//! its store functions accept.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// A value that did not match its sysfs format.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{what}: {reason} (got {value:?})")]
pub struct ParseError {
    pub what: &'static str,
    pub reason: String,
    pub value: String,
}

impl ParseError {
    fn new(what: &'static str, reason: impl Into<String>, value: &str) -> Self {
        Self { what, reason: reason.into(), value: value.to_owned() }
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// Colour

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const BLACK: Rgb = Rgb::new(0, 0, 0);
    pub const WHITE: Rgb = Rgb::new(255, 255, 255);

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Scales each channel by `factor` in 0.0..=1.0.
    pub fn scaled(self, factor: f32) -> Rgb {
        let f = factor.clamp(0.0, 1.0);
        let s = |c: u8| (f32::from(c) * f).round() as u8;
        Rgb::new(s(self.r), s(self.g), s(self.b))
    }

    /// Linear blend from `self` (t = 0) to `other` (t = 1).
    pub fn lerp(self, other: Rgb, t: f32) -> Rgb {
        let t = t.clamp(0.0, 1.0);
        let l = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * t).round() as u8;
        Rgb::new(l(self.r, other.r), l(self.g, other.g), l(self.b, other.b))
    }
}

impl FromStr for Rgb {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t = s.trim();
        let hex = t.strip_prefix('#').unwrap_or(t);
        if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(ParseError::new("colour", "expected 6 hex digits", s));
        }
        let ch = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).expect("validated hex");
        Ok(Rgb::new(ch(0), ch(2), ch(4)))
    }
}

/// Uppercase `RRGGBB`, the form the driver's store functions take.
impl fmt::Display for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

impl TryFrom<String> for Rgb {
    type Error = ParseError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl From<Rgb> for String {
    fn from(c: Rgb) -> String {
        c.to_string()
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// Animation mode

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnimationMode {
    #[default]
    Static,
    Breathing,
    Rainbow,
    Wave,
    Pulse,
    Chase,
    Sparkle,
    Candle,
    Aurora,
    Disco,
    Gradient,
    Spectrum,
}

impl AnimationMode {
    pub const ALL: [AnimationMode; 12] = [
        Self::Static,
        Self::Breathing,
        Self::Rainbow,
        Self::Wave,
        Self::Pulse,
        Self::Chase,
        Self::Sparkle,
        Self::Candle,
        Self::Aurora,
        Self::Disco,
        Self::Gradient,
        Self::Spectrum,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Breathing => "breathing",
            Self::Rainbow => "rainbow",
            Self::Wave => "wave",
            Self::Pulse => "pulse",
            Self::Chase => "chase",
            Self::Sparkle => "sparkle",
            Self::Candle => "candle",
            Self::Aurora => "aurora",
            Self::Disco => "disco",
            Self::Gradient => "gradient",
            Self::Spectrum => "spectrum",
        }
    }
}

impl FromStr for AnimationMode {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t = s.trim();
        Self::ALL
            .into_iter()
            .find(|m| m.as_str() == t)
            .ok_or_else(|| ParseError::new("animation mode", "unknown mode", s))
    }
}

impl fmt::Display for AnimationMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.as_str();
        // Capitalised for display in pick lists; `as_str` is the sysfs form.
        let mut c = s.chars();
        match c.next() {
            Some(first) => write!(f, "{}{}", first.to_ascii_uppercase(), c.as_str()),
            None => Ok(()),
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// Thermal profile

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThermalProfile {
    Silent,
    Normal,
    Performance,
}

impl ThermalProfile {
    pub const ALL: [ThermalProfile; 3] = [Self::Silent, Self::Normal, Self::Performance];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Silent => "silent",
            Self::Normal => "normal",
            Self::Performance => "performance",
        }
    }
}

impl FromStr for ThermalProfile {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t = s.trim();
        Self::ALL
            .into_iter()
            .find(|p| p.as_str() == t)
            .ok_or_else(|| ParseError::new("thermal profile", "unknown profile", s))
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// Fan curves

pub const CURVE_MIN_POINTS: usize = 2;
pub const CURVE_MAX_POINTS: usize = 8;
pub const CURVE_MAX_TEMP: u8 = 120;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurvePoint {
    pub temp: u8,
    pub percent: u8,
}

impl CurvePoint {
    pub const fn new(temp: u8, percent: u8) -> Self {
        Self { temp, percent }
    }
}

/// Parses `temp:percent` pairs. `(unset)` or an empty file is a curve with no points.
pub fn parse_curve(s: &str) -> Result<Vec<CurvePoint>, ParseError> {
    let t = s.trim();
    if t.is_empty() || t == "(unset)" {
        return Ok(Vec::new());
    }
    t.split_whitespace()
        .map(|pair| {
            let (temp, pct) =
                pair.split_once(':').ok_or_else(|| ParseError::new("fan curve", "expected temp:percent", pair))?;
            let temp = temp.parse::<u8>().map_err(|_| ParseError::new("fan curve", "bad temperature", pair))?;
            let percent = if pct.eq_ignore_ascii_case("max") {
                100
            } else {
                pct.parse::<u8>().map_err(|_| ParseError::new("fan curve", "bad percent", pair))?
            };
            Ok(CurvePoint { temp, percent })
        })
        .collect()
}

/// Checks the limits the driver enforces on a written curve.
pub fn validate_curve(points: &[CurvePoint]) -> Result<(), ParseError> {
    let shown = format_curve_unchecked(points);
    if !(CURVE_MIN_POINTS..=CURVE_MAX_POINTS).contains(&points.len()) {
        return Err(ParseError::new(
            "fan curve",
            format!("needs {CURVE_MIN_POINTS} to {CURVE_MAX_POINTS} points"),
            &shown,
        ));
    }
    if let Some(p) = points.iter().find(|p| p.temp > CURVE_MAX_TEMP || p.percent > 100) {
        return Err(ParseError::new(
            "fan curve",
            format!("point {}:{} out of range (0-{CURVE_MAX_TEMP} °C, 0-100 %)", p.temp, p.percent),
            &shown,
        ));
    }
    Ok(())
}

/// Validates and formats a curve for writing, sorted by temperature.
pub fn format_curve(points: &[CurvePoint]) -> Result<String, ParseError> {
    validate_curve(points)?;
    Ok(format_curve_unchecked(points))
}

fn format_curve_unchecked(points: &[CurvePoint]) -> String {
    let mut sorted = points.to_vec();
    sorted.sort_by_key(|p| p.temp);
    sorted.iter().map(|p| format!("{}:{}", p.temp, p.percent)).collect::<Vec<_>>().join(" ")
}

/// What `gpu_fan_curve` reads as.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpuCurve {
    FollowsCpu,
    Own(Vec<CurvePoint>),
}

pub fn parse_gpu_curve(s: &str) -> Result<GpuCurve, ParseError> {
    if s.trim() == "(cpu)" { Ok(GpuCurve::FollowsCpu) } else { parse_curve(s).map(GpuCurve::Own) }
}

// ---------------------------------------------------------------------------------------------------------------------
// Gradient config

pub const ZONE_COUNT: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GradientGroup {
    /// Zone indices 0..ZONE_COUNT, ascending, no duplicates.
    pub zones: Vec<u8>,
    pub colors: Vec<Rgb>,
}

pub fn parse_gradient_config(s: &str) -> Result<Vec<GradientGroup>, ParseError> {
    let t = s.trim();
    if t.is_empty() {
        return Ok(Vec::new());
    }
    t.split(';')
        .filter(|g| !g.trim().is_empty())
        .map(|group| {
            let (zones, colors) = group
                .split_once(':')
                .ok_or_else(|| ParseError::new("gradient config", "expected zones:colours", group))?;
            let mut zs = Vec::new();
            for z in zones.split(',') {
                let z: u8 = z.trim().parse().map_err(|_| ParseError::new("gradient config", "bad zone", group))?;
                if usize::from(z) >= ZONE_COUNT {
                    return Err(ParseError::new("gradient config", "zone must be 0-3", group));
                }
                if !zs.contains(&z) {
                    zs.push(z);
                }
            }
            zs.sort_unstable();
            let cs = colors.split(',').map(str::parse).collect::<Result<Vec<Rgb>, _>>()?;
            let g = GradientGroup { zones: zs, colors: cs };
            validate_gradient_group(&g).map_err(|reason| ParseError::new("gradient config", reason, group))?;
            Ok(g)
        })
        .collect()
}

/// Returns why a group cannot be written, if it cannot.
pub fn validate_gradient_group(g: &GradientGroup) -> Result<(), &'static str> {
    if g.zones.is_empty() {
        return Err("group has no zones");
    }
    if g.colors.is_empty() {
        return Err("group has no colours");
    }
    Ok(())
}

pub fn format_gradient_config(groups: &[GradientGroup]) -> String {
    groups
        .iter()
        .map(|g| {
            let zones = g.zones.iter().map(u8::to_string).collect::<Vec<_>>().join(",");
            let colors = g.colors.iter().map(Rgb::to_string).collect::<Vec<_>>().join(",");
            format!("{zones}:{colors}")
        })
        .collect::<Vec<_>>()
        .join(";")
}

// ---------------------------------------------------------------------------------------------------------------------
// Power limits

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum PowerLimits {
    Default,
    Unsupported,
    Custom { pl1: u32, pl2: u32 },
}

impl PowerLimits {
    /// Validates a user-entered pair.
    pub fn custom(pl1: u32, pl2: u32) -> Result<PowerLimits, String> {
        if pl1 == 0 || pl2 == 0 {
            return Err("PL1 and PL2 must be positive".into());
        }
        if pl2 < pl1 {
            return Err("PL2 must be at least PL1".into());
        }
        Ok(PowerLimits::Custom { pl1, pl2 })
    }
}

pub fn parse_power_limits(s: &str) -> Result<PowerLimits, ParseError> {
    match s.trim() {
        "default" | "0 0" => Ok(PowerLimits::Default),
        "unsupported" => Ok(PowerLimits::Unsupported),
        t => {
            let mut it = t.split_whitespace().map(str::parse::<u32>);
            match (it.next(), it.next(), it.next()) {
                (Some(Ok(pl1)), Some(Ok(pl2)), None) => Ok(PowerLimits::Custom { pl1, pl2 }),
                _ => Err(ParseError::new("power limits", "expected \"PL1 PL2\" or \"default\"", s)),
            }
        }
    }
}

/// Text to write for a value; `None` for `Unsupported`, which cannot be written.
pub fn format_power_limits(p: PowerLimits) -> Option<String> {
    match p {
        PowerLimits::Default => Some("default".into()),
        PowerLimits::Unsupported => None,
        PowerLimits::Custom { pl1, pl2 } => Some(format!("{pl1} {pl2}")),
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// Fan readings

/// `Ok(None)` for `n/a`.
pub fn parse_rpm(s: &str) -> Result<Option<u32>, ParseError> {
    match s.trim() {
        "n/a" => Ok(None),
        t => t.parse().map(Some).map_err(|_| ParseError::new("fan rpm", "expected a number or n/a", s)),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaxRpm {
    pub cpu: u32,
    pub gpu: u32,
}

/// `Ok(None)` for `unknown`.
pub fn parse_max_rpm(s: &str) -> Result<Option<MaxRpm>, ParseError> {
    let t = s.trim();
    if t == "unknown" {
        return Ok(None);
    }
    let mut it = t.split_whitespace().map(str::parse::<u32>);
    match (it.next(), it.next(), it.next()) {
        (Some(Ok(cpu)), Some(Ok(gpu)), None) => Ok(Some(MaxRpm { cpu, gpu })),
        _ => Err(ParseError::new("max rpm", "expected \"CPU GPU\" or unknown", s)),
    }
}

pub fn parse_bool01(s: &str) -> Result<bool, ParseError> {
    match s.trim() {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(ParseError::new("switch", "expected 0 or 1", s)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: Parse driver colour output
    #[test]
    fn parse_driver_colour_output() {
        assert_eq!("#ff8000\n".parse::<Rgb>().unwrap(), Rgb::new(255, 128, 0));
        assert_eq!("ff8000".parse::<Rgb>().unwrap(), Rgb::new(255, 128, 0));
    }

    /// Scenario: Reject malformed colour
    #[test]
    fn reject_malformed_colour() {
        for bad in ["#ff80", "12345G", "#ff80001", ""] {
            let err = bad.parse::<Rgb>().unwrap_err();
            assert_eq!(err.value, bad);
        }
    }

    /// Scenario: Format colour for writing
    #[test]
    fn format_colour_for_writing() {
        assert_eq!(Rgb::new(255, 128, 0).to_string(), "FF8000");
    }

    /// Scenario: Parse curve
    #[test]
    fn parse_curve_text() {
        let c = parse_curve("45:0 60:40 75:70 92:100\n").unwrap();
        assert_eq!(
            c,
            vec![CurvePoint::new(45, 0), CurvePoint::new(60, 40), CurvePoint::new(75, 70), CurvePoint::new(92, 100)]
        );
    }

    /// Scenario: Parse max keyword
    #[test]
    fn parse_max_keyword() {
        assert_eq!(parse_curve("40:0 92:max").unwrap()[1], CurvePoint::new(92, 100));
    }

    /// Scenario: Parse GPU following CPU
    #[test]
    fn parse_gpu_following_cpu() {
        assert_eq!(parse_gpu_curve("(cpu)\n").unwrap(), GpuCurve::FollowsCpu);
    }

    /// Scenario: Parse unset curve
    #[test]
    fn parse_unset_curve() {
        assert_eq!(parse_curve("(unset)\n").unwrap(), vec![]);
    }

    /// Scenario: Format curve sorted
    #[test]
    fn format_curve_sorted() {
        let pts = [CurvePoint::new(75, 70), CurvePoint::new(45, 0), CurvePoint::new(60, 40)];
        assert_eq!(format_curve(&pts).unwrap(), "45:0 60:40 75:70");
    }

    /// Scenario: Reject curve with too few or too many points
    #[test]
    fn reject_curve_with_too_few_or_too_many_points() {
        assert!(validate_curve(&[CurvePoint::new(40, 0)]).is_err());
        let nine: Vec<_> = (0..9).map(|i| CurvePoint::new(30 + i * 5, i * 10)).collect();
        assert!(validate_curve(&nine).is_err());
        assert!(validate_curve(&nine[..8]).is_ok());
    }

    /// Scenario: Reject curve point out of range
    #[test]
    fn reject_curve_point_out_of_range() {
        assert!(validate_curve(&[CurvePoint::new(40, 0), CurvePoint::new(90, 101)]).is_err());
        assert!(validate_curve(&[CurvePoint::new(40, 0), CurvePoint::new(121, 100)]).is_err());
    }

    /// Scenario: Parse gradient config
    #[test]
    fn parse_gradient_config_text() {
        let g = parse_gradient_config("0,1,2:FF0000,00FF00,0000FF;3:800080,FFA500").unwrap();
        assert_eq!(g.len(), 2);
        assert_eq!(g[0].zones, vec![0, 1, 2]);
        assert_eq!(g[0].colors, vec![Rgb::new(255, 0, 0), Rgb::new(0, 255, 0), Rgb::new(0, 0, 255)]);
        assert_eq!(g[1].zones, vec![3]);
        assert_eq!(g[1].colors, vec![Rgb::new(0x80, 0, 0x80), Rgb::new(0xFF, 0xA5, 0)]);
    }

    /// Scenario: Gradient config round trip
    #[test]
    fn gradient_config_round_trip() {
        let s = "0,1,2:FF0000,00FF00,0000FF;3:800080,FFA500";
        assert_eq!(format_gradient_config(&parse_gradient_config(s).unwrap()), s);
    }

    /// Scenario: Reject gradient zone out of range
    #[test]
    fn reject_gradient_zone_out_of_range() {
        assert!(parse_gradient_config("0,4:FF0000").is_err());
    }

    /// Scenario: Parse power limits
    #[test]
    fn parse_power_limits_text() {
        assert_eq!(parse_power_limits("65 90\n").unwrap(), PowerLimits::Custom { pl1: 65, pl2: 90 });
    }

    /// Scenario: Parse default power limits
    #[test]
    fn parse_default_power_limits() {
        assert_eq!(parse_power_limits("default\n").unwrap(), PowerLimits::Default);
    }

    /// Scenario: Reject PL2 below PL1
    #[test]
    fn reject_pl2_below_pl1() {
        let err = PowerLimits::custom(90, 65).unwrap_err();
        assert!(err.contains("PL2 must be at least PL1"));
    }

    /// Scenario: Parse unavailable RPM
    #[test]
    fn parse_unavailable_rpm() {
        assert_eq!(parse_rpm("n/a\n").unwrap(), None);
    }

    /// Scenario: Parse max RPM
    #[test]
    fn parse_max_rpm_text() {
        assert_eq!(parse_max_rpm("5200 5400\n").unwrap(), Some(MaxRpm { cpu: 5200, gpu: 5400 }));
    }
}
