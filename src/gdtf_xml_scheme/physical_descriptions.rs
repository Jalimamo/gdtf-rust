use serde::Deserialize;
use zip::ZipArchive;
use std::io::Cursor;
use gdtf_derive::GdtfVerify;
use crate::parsing_helpers::{
    ArrayOfColorCie, Ces, ColorCie, Float, Gender, InterpolationType, Percentage, Uint, empty_string_as_none
};
use crate::parsing_helpers::defaults::{
    default_weight, default_high_operating_temperature, default_connector_length, default_color_temperature,
    default_color_rendering_index, default_color_space_name, default_leg_height, default_low_operating_temperature
};
use crate::parsing_helpers::names_and_references::{Name, Named};
use crate::parsing_helpers::verification::{Verify, VerifySelf};


#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
#[verify(verify_self)]
pub struct PhysicalDescriptions {
    #[serde(rename = "Emitters", default)]
    pub emitters: Option<Emitters>,

    #[serde(rename = "Filters", default)]
    pub filters: Option<Filters>,

    #[serde(rename = "ColorSpace", default)]
    pub color_space: Option<ColorSpace>,

    #[serde(rename = "AdditionalColorSpaces", default)]
    pub additional_color_spaces: Option<ColorSpaces>,

    #[serde(rename = "Gamuts", default)]
    pub gamuts: Option<Gamuts>,

    #[serde(rename = "DMXProfiles", default)]
    pub dmx_profiles: Option<DmxProfiles>,

    #[serde(rename = "CRIs", default)]
    pub cris: Option<CRIs>,

    #[serde(rename = "Connectors", default)]
    pub connectors: Option<Connectors>,

    #[serde(rename = "Properties", default)]
    pub properties: Option<Properties>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
#[verify(uniqueness = emitters)]
pub struct Emitters {
    #[serde(rename = "Emitter", default)]
    pub emitters: Vec<Emitter>
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
#[verify(verify_self)]
pub struct Emitter {
    #[serde(rename = "@Name")]
    pub name: Name,

    #[serde(rename = "@Color", default)]
    pub color: Option<ColorCie>,

    #[serde(rename = "@DominantWaveLength", default)]
    pub dominant_wave_length: Option<Float>,

    #[serde(rename = "@DiodePart", deserialize_with = "empty_string_as_none", default)]
    pub diode_part: Option<String>,

    #[serde(rename = "Measurement", default)]
    pub measurements: Vec<Measurement>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
#[verify(uniqueness = filters)]
pub struct Filters {
    #[serde(rename = "Filter", default)]
    pub filters: Vec<Filter>
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
#[verify(verify_self)]
pub struct Filter {
    #[serde(rename = "@Name")]
    pub name: Name,

    #[serde(rename = "@Color")]
    pub color: ColorCie,

    #[serde(rename = "Measurement", default)]
    pub measurements: Vec<Measurement>,
}


#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct Measurement {
    // Should be bigger than 0 for additive color mixing
    #[serde(rename = "@Physical")]
    pub physical_unit: Percentage,

    // Should only be use with additive color mixing
    #[serde(rename = "@LuminousIntensity")]
    pub luminous_intensity: Option<Float>,

    // Should only be use with subtractive color mixing
    #[serde(rename = "@Transmission")]
    pub transmission: Option<Float>,

    #[serde(rename = "@InterpolationTo", default)]
    pub interpolation: InterpolationType,

    #[serde(rename = "MeasurementPoint", default)]
    pub measurement_points: Vec<MeasurementPoint>
}


#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct MeasurementPoint {
    #[serde(rename = "@WaveLength")]
    pub wave_length: Float,

    #[serde(rename = "@Energy")]
    pub energy: Float,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]

#[serde(deny_unknown_fields)]
pub struct ColorSpaces {
    #[serde(rename = "ColorSpace", default)]
    pub color_spaces: Vec<ColorSpace>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
pub struct ColorSpace {
    #[serde(rename = "@Name", default = "default_color_space_name")]
    pub name: Name,

    #[serde(flatten)]
    pub mode: ColorSpaceMode,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(untagged)]
pub enum ColorSpaceMode {
    Custom {
        #[serde(rename = "@Mode")]
        mode: OnlyCustomColorSpaceMode,

        #[serde(rename = "@Red")]
        red: ColorCie,

        #[serde(rename = "@Green")]
        green: ColorCie,

        #[serde(rename = "@Blue")]
        blue: ColorCie,

        #[serde(rename = "@WhitePoint")]
        white_point: ColorCie,
    },

    Standard {
        #[serde(rename = "@Mode", default)]
        mode: OtherColorSpaceMode,
    }
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]

pub enum OnlyCustomColorSpaceMode {
    Custom,
}

#[derive(Debug, Clone, Deserialize, Default, GdtfVerify)]
pub enum OtherColorSpaceMode {
    #[default]
    #[serde(rename = "sRGB")]
    SRGB,

    ProPhoto,
    ANSI,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
#[verify(uniqueness = gamuts)]
pub struct Gamuts {
    #[serde(rename = "Gamut", default)]
    pub gamuts: Vec<Gamut>
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct Gamut {
    #[serde(rename = "@Name")]
    pub name: Name,

    #[serde(rename = "@Points")]
    pub points: ArrayOfColorCie
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
#[verify(uniqueness = profiles)]
pub struct DmxProfiles {
    #[serde(rename = "DMXProfile", default)]
    pub profiles: Vec<DmxProfile>
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct DmxProfile {
    #[serde(rename = "@Name")]
    pub name: Name,

    #[serde(rename = "Point", default)]
    pub points: Vec<DmxProfilePoint>
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct DmxProfilePoint {
    #[serde(rename = "@DMXPercentage", default)]
    pub dmx_percentage: Percentage,

    #[serde(rename = "@CFC0", default)]
    pub cf0: Float,

    #[serde(rename = "@CFC1", default)]
    pub cf1: Float,

    #[serde(rename = "@CFC2", default)]
    pub cf2: Float,

    #[serde(rename = "@CFC3", default)]
    pub cf3: Float,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]

#[serde(deny_unknown_fields)]
pub struct CRIs {
    #[serde(rename = "CRIGroup", default)]
    pub cri_groups: Vec<CRIGroup>
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct CRIGroup {
    #[serde(rename = "@ColorTemperature", default = "default_color_temperature")]
    pub color_temperature: Float,

    #[serde(rename = "CRI", default)]
    pub cris: Vec<CRI>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct CRI {
    #[serde(rename = "@CES", default)]
    pub ces: Ces,

    #[serde(rename = "@ColorRenderingIndex", default = "default_color_rendering_index")]
    pub color_rendering_index: u8,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct Connectors {
    #[serde(rename = "Connector", default)]
    pub connectors: Vec<Connector>
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct Connector {
    #[serde(rename = "@Name")]
    pub name: Name,

    #[serde(rename = "@Type")]
    pub connector_type: Name,

    #[serde(rename = "@DMXBreak", default)]
    pub  dmx_break: Option<Uint>,

    #[serde(rename = "@Gender", default)]
    pub  gender: Gender,

    #[serde(rename = "@Length", default = "default_connector_length")]
    pub length: Float,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct Properties {
    #[serde(rename = "OperatingTemperature", default)]
    pub  operating_temperature: Option<OperatingTemperature>,

    #[serde(rename = "Weight", default)]
    pub weight: Option<Weight>,

    #[serde(rename = "PowerConsumption", default)]
    power_consumption: Vec<PowerConsumption>,

    #[serde(rename = "LegHeight", default)]
    pub leg_height: Option<LegHeight>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct OperatingTemperature {
    #[serde(rename = "@Low", default = "default_low_operating_temperature")]
    pub low: Float,

    #[serde(rename = "@High", default = "default_high_operating_temperature")]
    pub high: Float,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct Weight {
    #[serde(rename = "@Value", default = "default_weight")]
    pub  weight: Float,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]

pub struct PowerConsumption {
    // Not further explained in the DIN Spec 15800
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]

#[serde(deny_unknown_fields)]
pub struct LegHeight {
    #[serde(rename = "@Value", default = "default_leg_height")]
    pub leg_height: Float,
}
