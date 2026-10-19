use serde::Deserialize;
use zip::ZipArchive;
use std::io::Cursor;
use gdtf_derive::GdtfVerify;
use crate::parsing_helpers::verification::Verify;

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct FTPresets {
    #[serde(rename = "FTPreset", default)]
    pub ft_presets: Vec<FTPreset>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct FTPreset {
    // Unused, but I wanted to parse it anyway
}