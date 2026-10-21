pub mod names_and_references;
pub mod resources;
pub mod defaults;
pub mod verification;
mod dmx_verification;

use std::any::Any;
use std::fmt::{self, Display, Formatter};
use std::io::Cursor;
use std::ops::Add;
use serde::{Deserialize, Deserializer};
use chrono::NaiveDateTime;
use crate::parsing_helpers::DMXAddress::{Absolute, UniverseAddress};
use crate::parsing_helpers::DmxValue::{Mirroring, Shifting};
use arrayvec::ArrayVec;
use zip::ZipArchive;
use gdtf_derive::GdtfVerify;
use crate::parsing_helpers::names_and_references::Name;
use crate::parsing_helpers::resources::{Resource, ResourceType};
use verification::Verify;

pub type Uint = u64;
pub type Int = i64;

pub const DMX_UNIVERSE_SIZE: u32 = 512;

pub fn deserialize_hex<'de, D>(deserializer: D) -> Result<u64, D::Error> where D: Deserializer<'de> {
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        Some(s) if !s.trim().is_empty() => {
            let clean_hex = s.trim_start_matches("0x").trim_start_matches("0x");
            u64::from_str_radix(clean_hex, 16).map_err(serde::de::Error::custom)
        }
        _ => Ok(0), //Default value
    }
}

pub type Float = f64;

#[derive(Debug, Clone, PartialEq)]
pub struct GdtfDate(pub NaiveDateTime);

impl<'de> Deserialize<'de> for GdtfDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        let parsed = NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S")
            .map_err(serde::de::Error::custom)?;
        Ok(GdtfDate(parsed))
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct ColorCie {
    pub x: f64,
    pub y: f64,
    pub yy: f64,
}

impl <'de> Deserialize<'de> for ColorCie {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        let parts: Vec<&str> = s.split(',').collect();

        if parts.len() != 3 {
            return Err(serde::de::Error::custom(
                format!("Color-Cie must contain exactly 3 comma-separated floats, got: {}", s)
            ));
        }

        let x = parts[0].trim().parse().map_err(serde::de::Error::custom)?;
        let y = parts[1].trim().parse().map_err(serde::de::Error::custom)?;
        let uppercase_y = parts[2].trim().parse().map_err(serde::de::Error::custom)?;

        Ok(ColorCie { x, y, yy: uppercase_y })
    }
}

#[derive(Debug, Clone, PartialEq, GdtfVerify)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl <'de> Deserialize<'de> for Vector3 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        let s_cleaned = s.replace("{", "").replace("}", "");
        let parts: Vec<&str> = s_cleaned.split(',').collect();

        if parts.len() != 3 {
            return Err(serde::de::Error::custom("Vector3 must have exactly 3 floats"));
        }

        let x = parts[0].trim().parse().map_err(serde::de::Error::custom)?;
        let y = parts[1].trim().parse().map_err(serde::de::Error::custom)?;
        let z = parts[2].trim().parse().map_err(serde::de::Error::custom)?;

        Ok(Vector3 { x, y, z })
    }
}

#[derive(Debug, Clone, PartialEq, GdtfVerify)]
pub struct Matrix {
    pub rows: [[f64; 4]; 4]
}

impl <'de> Deserialize<'de> for Matrix {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;

        let rows = parse_gdtf_matrix::<4,4>(&s).map_err(serde::de::Error::custom)?;

        Ok(Matrix { rows })
    }
}


#[derive(Debug, Clone, PartialEq, GdtfVerify)]
pub struct Rotation {
    pub rows: [[f64; 3]; 3],
}

impl <'de> Deserialize<'de> for Rotation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;

        let rows = parse_gdtf_matrix::<3,3>(&s).map_err(serde::de::Error::custom)?;

        Ok(Rotation { rows })
    }
}

//Enums
pub fn yes_no_bool<'de, D>(deserializer: D) -> Result<bool, D::Error> where D: Deserializer<'de> {
    let s = String::deserialize(deserializer)?;
    match s.trim().to_lowercase().as_str() {
        "yes"   | "true"    | "on"  | "1"   => Ok(true),
        "no"    | "false"   | "off" | "0"   => Ok(false),
        other => Err(serde::de::Error::custom(format!("Invalid Yes/No-Value: {}", other))),
    }
}

// pub fn optional_yes_no_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error> where D: Deserializer<'de> {
//     let opt = Option::<String>::deserialize(deserializer)?;
//     match opt {
//         None => Ok(None),
//         Some(s) => {
//             match s.trim().to_lowercase().as_str() {
//                 "yes"   | "true"    | "1"   => Ok(Some(true)),
//                 "no"    | "false"   | "0"   => Ok(Some(false)),
//                 other => Err(serde::de::Error::custom(format!("Invalid Yes/No-Value: {}", other))),
//             }
//         }
//     }
// }

#[derive(Debug, Copy, Clone, PartialEq, GdtfVerify, Eq, Hash)]
pub enum DMXAddress {
    Absolute(u32),
    UniverseAddress { universe: u16, address: u16 },
}

impl Add for DMXAddress {
    type Output = DMXAddress;

    fn add(self, rhs: Self) -> Self::Output {
        DMXAddress::Absolute(
            self.as_absolute()
                .checked_add(rhs.as_absolute().saturating_sub(1))
                .expect("DMXAddress addition overflowed"),
        )
    }
}

impl Add<u32> for DMXAddress {
    type Output = DMXAddress;

    fn add(self, rhs: u32) -> Self::Output {
        DMXAddress::Absolute(
            self.as_absolute()
                .checked_add(rhs.saturating_sub(1))
                .expect("DMXAddress addition overflowed"),
        )
    }
}

impl <'de> Deserialize<'de> for DMXAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        if s.contains('.') {
            let parts: Vec<&str> = s.split('.').collect();
            if parts.len() != 2 {
                return Err(serde::de::Error::custom(format!("Invalid DMX-Address format: {}", s)));
            }
            let universe = parts[0].parse().map_err(serde::de::Error::custom)?;
            let address = parts[1].parse().map_err(serde::de::Error::custom)?;
            Ok(UniverseAddress { universe, address })
        } else {
            let abs = s.parse().map_err(serde::de::Error::custom)?;
            Ok(Absolute(abs))
        }
    }
}

impl DMXAddress {
    fn as_absolute(self) -> u32 {
        match self {
            DMXAddress::Absolute(abs) => abs,
            DMXAddress::UniverseAddress {universe, address} => {
                (u32::from(universe) - 1) * DMX_UNIVERSE_SIZE +  u32::from(address)
            },
        }
    }
    
    fn from_u32(u: u32) -> DMXAddress {
        DMXAddress::Absolute(u)
    }
}


#[derive(Debug, Clone, PartialEq, GdtfVerify)]
pub enum DmxValue {
    Mirroring {value: u32, byte_count: u8},
    Shifting {value: u32, byte_count: u8},
}

impl std::str::FromStr for DmxValue {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid DMX-Value format: {}", s));
        }

        let value = parts[0]
            .parse::<u32>()
            .map_err(|e| e.to_string())?;

        let is_shifting = parts[1].ends_with('s');

        let bytes_str = if is_shifting {
            parts[1].trim_end_matches('s')
        } else {
            parts[1]
        };

        let byte_count = bytes_str.parse::<u8>().map_err(|e| e.to_string())?;

        if is_shifting {
            Ok(Shifting { value, byte_count })
        } else {
            Ok(Mirroring { value, byte_count })
        }
    }
}


impl <'de> Deserialize<'de> for DmxValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, GdtfVerify)]
pub struct DataVersion {
    pub major: Uint,
    pub minor: Uint,
}

impl<'de> Deserialize<'de> for DataVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 2 {
            return Err(serde::de::Error::custom(
                format!("Invalid DataVersion format. Expected 'Uint.Uint', got: '{}'", s)
            ));
        }

        let major = parts[0].parse().map_err(serde::de::Error::custom)?;
        let minor = parts[1].parse().map_err(serde::de::Error::custom)?;

        Ok(DataVersion { major, minor })
    }
}

#[derive(Debug, Clone, PartialEq, GdtfVerify)]
pub struct ArrayOf2Float([Float; 2]);

impl<'de> Deserialize<'de> for ArrayOf2Float {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 2 {
            return Err(serde::de::Error::custom(
                format!("Invalid ArrayOfFloat format. Expected 'Float,Float', got: '{}'", s)
            ));
        }

        let x = parts[0].trim().parse().map_err(serde::de::Error::custom)?;
        let y = parts[1].trim().parse().map_err(serde::de::Error::custom)?;

        Ok(ArrayOf2Float([x, y]))
    }
}


#[derive(Debug, Clone, PartialEq, GdtfVerify)]
pub struct Percentage(pub f64);

impl<'de> Deserialize<'de> for Percentage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;

        let percentage = s.trim().parse().map_err(serde::de::Error::custom)?;

        if percentage < 0.0 || percentage > 100.0 {
            return Err(serde::de::Error::custom(
                format!("Invalid Range. Percentage must be <= 0 and >= 100. Got '{}'", s)
            ));
        }

        Ok(Percentage(percentage))

    }
}

impl Default for Percentage {
    fn default() -> Self {
        Percentage(0.0)
    }
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Default, GdtfVerify)]
pub enum PhysicalUnit {
    #[default]
    None,
    Percent,
    Length,
    Mass,
    Time,
    Temperature,
    LuminousIntensity,
    Angle,
    Force,
    Frequency,
    Current,
    Voltage,
    Power,
    Energy,
    Area,
    Volume,
    Speed,
    Acceleration,
    AngularSpeed,
    AngularAccc, //Possible typo by the GDTF-Guys, but we follow them to hell and back we would
    WaveLength,
    ColorComponent,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, GdtfVerify)]
#[serde(rename = "Type")]
pub enum SubPhysicalUnitType {
    PlacementOffset,
    Amplitude,
    AmplitudeMin,
    AmplitudeMax,
    Duration,
    DutyCycle,
    TimeOffset,
    MinimumOpening,
    Value,
    RatioHorizontal,
    RatioVertical,
}

impl Display for SubPhysicalUnitType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq, Default, GdtfVerify)]
pub enum InterpolationType {
    #[default]
    Linear,
    Step,
    Log,
}

#[derive(Debug, Clone, PartialEq, Default, GdtfVerify)]
pub struct ArrayOfColorCie(Vec<ColorCie>);

impl<'de> Deserialize<'de> for ArrayOfColorCie {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        let cleaned = s.replace("{", "");
        let blocks = cleaned.split('}').filter(|b| !b.trim().is_empty());

        let mut array_of_colors = vec![];

        for block in blocks {
            let values: Vec<&str> = block.split(',').collect();
            if values.len() != 3 {
                return Err(serde::de::Error::custom(
                    format!("Invalid Number of Cie-Arguments. Needs 3 (x, y, Y), got {}", values.len())
                ));
            }

            let x = values[0].trim().parse().map_err(serde::de::Error::custom)?;
            let y = values[1].trim().parse().map_err(serde::de::Error::custom)?;
            let uppercase_y = values[2].trim().parse().map_err(serde::de::Error::custom)?;

            let color = ColorCie {
                x,y,
                yy: uppercase_y,
            };

            array_of_colors.push(color);
        }

        Ok(ArrayOfColorCie(array_of_colors))
    }
}

#[derive(Debug, Clone, PartialEq, GdtfVerify)]
pub struct Ces(u8);

impl<'de> Deserialize<'de> for Ces {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        let s = s.trim();

        if s.len() != 5 || !s.starts_with("CES") {
            return Err(serde::de::Error::custom(
                format!("Invalid CES format: '{}'. Expected format 'CESxx' (e.g., 'CES01').", s)
            ))
        }

        let num_str = &s[3..];
        let num = num_str.parse().map_err(serde::de::Error::custom)?;

        if !(1..=99).contains(&num) {
            return Err(serde::de::Error::custom(
                format!("CES number out of bounds: {}. Must be between 1 and 99", s)
            ))
        }

        Ok(Ces(num))
    }
}

impl Default for Ces {
    fn default() -> Self {
        Ces(01)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Gender(Int);

impl Default for Gender {
    fn default() -> Self {
        Gender(0)
    }
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize, GdtfVerify)]
pub enum PrimitiveModelType {
    #[default]
    Undefined,
    Cube,
    Cylinder,
    Sphere,
    Base,
    Yoke,
    Head,
    Scanner,
    Conventional,
    Pigtail,
    Base1_1,
    Scanner1_1,
    Conventional1_1,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize, GdtfVerify)]
pub enum LampType {
    #[default]
    Discharge,
    Tungsten,
    Halogen,
    LED,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize, GdtfVerify)]
pub enum BeamType {
    #[default]
    Wash,
    Spot,
    None,
    Rectangle,
    PC,
    Fresnel,
    Glow,
}


#[derive(Debug, Clone, PartialEq, Default, Deserialize, GdtfVerify)]
pub enum LaserColorType {
    #[default]
    RGB,
    SingleWaveLength
}

#[derive(Debug, Clone, PartialEq, Deserialize, GdtfVerify)]
pub enum WiringObjectComponentType {
    Input,
    Output,
    PowerSource,
    Consumer,
    Fuse,
    NetworkProvider,
    NetworkInput,
    NetworkOutput,
    NetworkInOut,
}

#[derive(Debug, Clone, PartialEq, Deserialize, GdtfVerify)]
pub enum FuseRating {
    B, C, D, K, Z,
}

#[derive(Debug, Clone, PartialEq, Deserialize, GdtfVerify)]
pub enum WiringOrientation {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone, PartialEq, Deserialize, GdtfVerify)]
pub enum StructureType {
    CenterLineBased,
    Detail,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
pub enum StructureCrossSectionType {
    TrussFramework,
    Tube
}

#[derive(Debug, Clone, PartialEq, Deserialize, GdtfVerify)]
pub enum SupportType {
    GroundSupport,
    Rope
}


#[derive(Debug, Clone, PartialEq, GdtfVerify, Eq, Hash)]
pub enum DMXBreak {
    Overwrite,
    Value(u32)
}

impl DMXBreak {
    pub fn get_value(&self) -> Option<u32> {
        match self {
            DMXBreak::Overwrite => None,
            DMXBreak::Value(value) => Some(*value)
        }
    } 
}

impl<'de> Deserialize<'de> for DMXBreak {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        if s.eq_ignore_ascii_case("overwrite") {
            Ok(DMXBreak::Overwrite)
        } else {
            s.parse::<u32>()
                .map(DMXBreak::Value)
                .map_err(|_| serde::de::Error::custom(format!("Invalid DMXBreak value: {}", s)))
        }
    }
}

#[derive(Debug, Clone, PartialEq, GdtfVerify)]
pub enum SemiUniqueBreak {
    Overwrite(u32),
    UniqueValue(u32),
}

impl <'de> Deserialize<'de> for SemiUniqueBreak {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        s.parse::<u32>()
            .map(SemiUniqueBreak::UniqueValue)
            .map_err(|_| serde::de::Error::custom(format!("Invalid SemiUniqueBreak value: {}", s)))
    }
}

impl Display for SemiUniqueBreak {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl SemiUniqueBreak {
    pub fn get_value(&self) -> u32 {
        match self {
            SemiUniqueBreak::Overwrite(value) => *value,
            SemiUniqueBreak::UniqueValue(value) => *value,
        }
    }

    pub fn to_overwrite(&mut self) {
        match self {
            SemiUniqueBreak::Overwrite(_) => {},
            SemiUniqueBreak::UniqueValue(value) => *self = SemiUniqueBreak::Overwrite(value.clone()),
        }
    }

    pub fn as_dmx_break(&self) -> DMXBreak {
        match self {
            SemiUniqueBreak::Overwrite(_) => DMXBreak::Overwrite,
            SemiUniqueBreak::UniqueValue(x) => DMXBreak::Value(*x),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DMXOffset(ArrayVec<u32, 4>);

impl<'de> Deserialize<'de> for DMXOffset {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;

        if s.eq_ignore_ascii_case("none") || s.is_empty() {
            return Ok(DMXOffset(ArrayVec::new()));
        }

        let offsets: Result<ArrayVec<u32, 4>, D::Error> = s
            .split(',')
            .map(|val| {
                val.trim()
                    .parse::<u32>()
                    .map_err(|_| {
                        serde::de::Error::custom(
                            format!("Invalid Offset int: {}", val.trim())
                        )
                    })
            })
            .collect();

        Ok(DMXOffset(offsets?))
    }
}

impl IntoIterator for DMXOffset {
    type Item = u32;
    type IntoIter = arrayvec::IntoIter<u32, 4>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a DMXOffset {
    type Item = &'a u32;
    type IntoIter = std::slice::Iter<'a, u32>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[derive(Debug, Clone, PartialEq, GdtfVerify)]
pub struct HighlightValue(Option<DmxValue>);

impl<'de> Deserialize<'de> for HighlightValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        if s.eq_ignore_ascii_case("None") {
            Ok(HighlightValue(None))
        } else {
            Ok(HighlightValue(Some(
                s.parse::<DmxValue>().map_err(serde::de::Error::custom)?
            )))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize, GdtfVerify)]
pub enum RespondToMaster {
    #[default]
    None,
    Grand,
    Group,
}

#[derive(Debug, Clone, PartialEq, Deserialize, GdtfVerify)]
pub enum DMXRelationType {
    Multiply,
    Override,
}

pub fn empty_string_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error> where D: Deserializer<'de> {
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(opt.filter(|s| !s.trim().is_empty()))
}

pub fn empty_name_as_none<'de, D>(deserializer: D) -> Result<Option<Name>, D::Error> where D: Deserializer<'de> {
    let opt = Option::<Name>::deserialize(deserializer)?;
    Ok(opt.filter(|name| !name.as_str().trim().is_empty()))
}

pub fn empty_resource_as_none<'de, D, T: ResourceType>(deserializer: D)
    -> Result<Option<Resource<T>>, D::Error> where D: Deserializer<'de>
{
    let opt = Option::<Resource<T>>::deserialize(deserializer)?;
    Ok(opt.filter(|res| !res.0.trim().is_empty()))
}

pub fn zero_as_none<'de, D>(deserializer: D) -> Result<Option<Uint>, D::Error> where D: Deserializer<'de> {
    let opt = Option::<Uint>::deserialize(deserializer)?;
    Ok(opt.filter(|value| *value != Uint::from(0u64)))
}

fn parse_gdtf_matrix<const ROWS: usize, const COLS: usize>(s: &str) -> Result<[[f64; COLS]; ROWS], String> {
    let cleaned = s.replace("{", "");

    let mut blocks = cleaned.split('}').filter(|b| !b.trim().is_empty());

    let mut matrix = [[0.0; COLS]; ROWS];

    for r in 0..ROWS {
        let block = blocks
            .next()
            .ok_or_else(|| format!("Missing matrix block for row {}", r))?;

        let parts: Vec<&str> = block.split(',').collect();
        if parts.len() != COLS {
            return Err(format!("Row {} must contain exactly {} floats, got {}", r, COLS, parts.len()));
        }

        for c in 0..COLS {
            matrix[r][c] = parts[c]
                .trim()
                .parse()
                .map_err(|e| format!("Float parse error at [{}, {}]: {}", r, c, e))?;
        }
    }
    Ok(matrix)
}

fn is_valid_gdtf_name_char(c: char) -> bool {
    if c as u32 > 127 {
        return true;
    }

    matches!(
        c,
        ' ' | '"' | '#' | '%' | '\'' | '(' | ')' | '*' | '+' | '-' | '/' |
        '0'..='9' | ':'..='>' | '@'..='Z' | '_' | '`' | 'a'..='z'
    ) || is_invalid_but_gdtf_name_we_allow_it(c)
}

//Deviation: Not allowed, but some stupid fucks want to use it anyway. Fuck you!
fn is_invalid_but_gdtf_name_we_allow_it(c: char) -> bool {
    matches!(
        c,
        '[' | ']' | '.' | ',' | '&' | '!'
    )
}