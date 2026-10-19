use serde::Deserialize;
use zip::ZipArchive;
use std::io::Cursor;
use gdtf_derive::GdtfVerify;
use crate::parsing_helpers::{empty_resource_as_none, Float, PrimitiveModelType};
use crate::parsing_helpers::names_and_references::{Name, Named};
use crate::parsing_helpers::resources::{Resource, Model3DFile};
use crate::parsing_helpers::verification::Verify;


#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
#[verify(uniqueness = models)]
pub struct Models {
    #[serde(rename = "Model", default)]
    pub models: Vec<Model>
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct Model {
    #[serde(rename = "@Name")]
    pub name: Name,

    #[serde(rename = "@Length", default)]
    pub length: Float,

    #[serde(rename = "@Width", default)]
    pub width: Float,

    #[serde(rename = "@Height", default)]
    pub height: Float,

    #[serde(rename = "@PrimitiveType", default)]
    pub primitive_type: PrimitiveModelType,

    #[serde(rename = "@File", default, deserialize_with = "empty_resource_as_none")]
    pub file: Option<Resource<Model3DFile>>,

    #[serde(rename = "@SVGOffsetX", default)]
    pub svg_offset_x: Float,

    #[serde(rename = "@SVGOffsetY", default)]
    pub svg_offset_y: Float,

    #[serde(rename = "@SVGSideOffsetX", default)]
    pub svg_side_offset_x: Float,

    #[serde(rename = "@SVGSideOffsetY", default)]
    pub svg_side_offset_y: Float,

    #[serde(rename = "@SVGFrontOffsetX", default)]
    pub svg_front_offset_x: Float,

    #[serde(rename = "@SVGFrontOffsetY", default)]
    pub svg_front_offset_y: Float,
}