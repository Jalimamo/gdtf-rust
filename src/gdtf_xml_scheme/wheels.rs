use zip::ZipArchive;
use std::io::Cursor;
use serde::Deserialize;
use gdtf_derive::GdtfVerify;
use crate::parsing_helpers::{ArrayOf2Float, ColorCie, Float, Rotation};
use crate::parsing_helpers::defaults::default_color_cie;
use crate::parsing_helpers::names_and_references::{Name, OptionalNodePath, Named};
use crate::parsing_helpers::resources::{Resource, WheelPNG};
use crate::parsing_helpers::verification::{Verify};
use crate::gdtf_xml_scheme::physical_descriptions::Filter;

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
#[verify(uniqueness = wheels)]
pub struct Wheels {
    #[serde(rename = "Wheel", default)]
    pub wheels: Vec<Wheel>
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct Wheel {
    #[serde(rename = "@Name")]
    pub name: Name,

    #[serde(rename = "Slot", default)]
    pub wheel_slots: Vec<WheelSlot>
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct WheelSlot {
    #[serde(rename = "@Name")]
    pub name: Name,

    #[serde(rename = "@Color", default = "default_color_cie")]
    pub color: ColorCie,

    #[serde(rename = "@Filter", default)]
    pub filters: OptionalNodePath<Filter>,

    #[serde(rename = "@MediaFileName", default)]
    pub media_file_name: Option<Resource<WheelPNG>>,

    #[serde(rename = "Facet", default)]
    pub prism_facets: Vec<PrismFacet>,

    #[serde(rename = "AnimationSystem", default)]
    pub animation_wheel: Option<AnimationSystem>
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct PrismFacet {
    #[serde(rename = "@Color", default = "default_color_cie")]
    color: ColorCie,

    #[serde(rename = "@Rotation")]
    rotation: Rotation,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct AnimationSystem {
    #[serde(rename = "@P1")]
    p1: ArrayOf2Float,

    #[serde(rename = "@P2")]
    p2: ArrayOf2Float,

    #[serde(rename = "@P3")]
    p3: ArrayOf2Float,

    #[serde(rename = "@Radius")]
    pub radius: Float,
}