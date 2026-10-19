use serde::Deserialize;
use zip::ZipArchive;
use std::io::Cursor;
use gdtf_derive::GdtfVerify;
use crate::parsing_helpers::{
    ColorCie, Float, PhysicalUnit, SubPhysicalUnitType,
};
use crate::parsing_helpers::names_and_references::{Name, Named, NodePath, OptionalNodePath};
use crate::parsing_helpers::verification::Verify;

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct AttributeDefinitions {
    #[serde(rename = "ActivationGroups", default)]
    pub activation_groups: Option<ActivationGroups>,

    #[serde(rename = "FeatureGroups")]
    pub feature_groups: FeatureGroups,

    #[serde(rename = "Attributes")]
    pub attributes: Attributes,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
#[verify(uniqueness = activation_group)]
pub struct ActivationGroups {
    #[serde(rename = "ActivationGroup", default)]
    pub activation_group: Vec<ActivationGroup>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct ActivationGroup {
    #[serde(rename = "@Name")]
    pub name: Name,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
#[verify(uniqueness = feature_group)]
pub struct FeatureGroups {
    #[serde(rename = "FeatureGroup", default)]
    pub feature_group: Vec<FeatureGroup>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
#[verify(uniqueness = features)]
pub struct FeatureGroup {
    #[serde(rename = "@Name")]
    pub name: Name,

    #[serde(rename = "@Pretty")]
    pub pretty: String,

    #[serde(rename = "Feature", default)]
    pub features: Vec<Feature>
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct Feature {
    #[serde(rename = "@Name")]
    pub name: Name,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
#[verify(uniqueness = attributes)]
pub struct Attributes {
    #[serde(rename = "Attribute", default)]
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct Attribute {
    #[serde(rename = "@Name")]
    pub name: Name,

    #[serde(rename = "@Pretty")]
    pub pretty: String,

    #[serde(rename = "@ActivationGroup", default)]
    pub activation_group: OptionalNodePath<ActivationGroup>,

    #[serde(rename = "@Feature")]
    pub feature: NodePath<Feature>,

    #[serde(rename = "@MainAttribute", default)]
    pub main_attribute: OptionalNodePath<Attribute>,

    #[serde(rename = "@PhysicalUnit", default)]
    pub physical_unit: PhysicalUnit,

    #[serde(rename = "@Color", default)]
    pub color: Option<ColorCie>,

    #[serde(rename = "SubPhysicalUnit", default)]
    pub sub_physical_unit: Vec<SubPhysicalUnit>,


}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct SubPhysicalUnit {
    #[serde(rename = "@Type")]
    pub unit_type: SubPhysicalUnitType,

    #[serde(rename = "@PhysicalUnit", default)]
    pub physical_unit: PhysicalUnit,

    #[serde(rename = "@PhysicalFrom")]
    pub physical_from: Float,

    #[serde(rename = "@PhysicalTo")]
    pub physical_to: Float,
}