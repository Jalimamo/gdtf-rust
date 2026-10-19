 pub mod attribute_definitions; pub mod wheels; pub mod physical_descriptions; pub mod models; pub mod geometries;
pub mod dmx_modes; pub mod revisions; pub mod fixture_type_presets; pub mod supported_protocols;

use zip::ZipArchive;
use std::io::Cursor;
use crate::parsing_helpers::*;
use serde::Deserialize;
use gdtf_derive::GdtfVerify;
use crate::gdtf_xml_scheme::attribute_definitions::AttributeDefinitions;
use crate::gdtf_xml_scheme::dmx_modes::DMXModes;
use crate::gdtf_xml_scheme::fixture_type_presets::FTPresets;
use crate::gdtf_xml_scheme::geometries::Geometries;
use crate::gdtf_xml_scheme::models::Models;
use crate::gdtf_xml_scheme::physical_descriptions::PhysicalDescriptions;
use crate::gdtf_xml_scheme::revisions::Revisions;
use crate::gdtf_xml_scheme::supported_protocols::Protocols;
use crate::gdtf_xml_scheme::wheels::Wheels;
use crate::parsing_helpers::resources::{Resource, RootPNG};
use crate::parsing_helpers::verification::Verify;

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(rename = "GDTF")]
#[serde(deny_unknown_fields)]
pub struct Gdtf {
    #[serde(rename = "@DataVersion")]
    pub data_version: DataVersion,
    #[serde(rename = "FixtureType")]
    pub fixture_type: FixtureType,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[verify(push_to_roots = update)]
// #[serde(deny_unknown_fields)]
pub struct FixtureType {
    #[serde(rename = "@Name")]
    pub name: String,

    #[serde(rename = "@ShortName", deserialize_with = "empty_string_as_none", default)]
    pub short_name: Option<String>,

    #[serde(rename = "@LongName", deserialize_with = "empty_string_as_none", default)]
    pub long_name: Option<String>,

    #[serde(rename = "@Manufacturer")]
    pub manufacturer: String,

    #[serde(rename = "@Description", deserialize_with = "empty_string_as_none", default)]
    pub description: Option<String>,

    #[serde(rename = "@FixtureTypeID")]
    pub fixture_type_id: String,

    #[serde(rename = "@Thumbnail", deserialize_with = "empty_resource_as_none", default)]
    pub thumbnail: Option<Resource<RootPNG>>,

    #[serde(rename = "@ThumbnailOffsetX")]
    pub thumbnail_offset_x: Option<Int>,

    #[serde(rename = "@ThumbnailOffsetY")]
    pub thumbnail_offset_y: Option<Int>,

    #[serde(rename = "@RefFT", deserialize_with = "empty_string_as_none", default)]
    pub ref_ft: Option<String>,

    #[serde(rename = "@CanHaveChildren", deserialize_with = "yes_no_bool", default)]
    pub can_have_children: bool,



    #[serde(rename = "AttributeDefinitions")]
    pub attribute_definitions: AttributeDefinitions,

    #[serde(rename = "Wheels", default)]
    pub wheels: Option<Wheels>,

    #[serde(rename = "PhysicalDescriptions", default)]
    pub physical_descriptions: Option<PhysicalDescriptions>,

    #[serde(rename = "Models", default)]
    pub models: Option<Models>,

    #[serde(rename = "Geometries")]
    pub geometries: Geometries,

    #[serde(rename = "DMXModes")]
    pub dmx_modes: DMXModes,

    #[serde(rename = "Revisions", default)]
    pub revisions: Option<Revisions>,

    #[serde(rename = "FTPresets", default)]
    pub ft_presets: Option<FTPresets>,

    #[serde(rename = "Protocols", default)]
    pub protocols: Option<Protocols>,
}
