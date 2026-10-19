use serde::Deserialize;
use zip::ZipArchive;
use std::io::Cursor;
use gdtf_derive::GdtfVerify;
use crate::parsing_helpers::{deserialize_hex, Uint};
use crate::parsing_helpers::names_and_references::NodePath;
use crate::parsing_helpers::verification::Verify;
use crate::gdtf_xml_scheme::dmx_modes::DMXMode;


#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct Protocols {
    #[serde(rename = "FTRDM", default)]
    pub rdm: Option<RDMProtocol>,
    #[serde(rename = "Art-Net", default)]
    pub art_net: Option<ArtNetProtocol>,
    #[serde(rename = "sACN", default)]
    pub sacn: Option<SACNProtocol>,
    #[serde(rename = "PosiStageNet", default)]
    pub posi_stage_net: Option<PosiStageNetProtocol>,
    #[serde(rename = "OpenSoundControl", default)]
    pub open_sound_control: Option<OpenSoundControlProtocol>,
    #[serde(rename = "CITP", default)]
    pub citp: Option<CITPProtocol>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct RDMProtocol {
    #[serde(rename = "@ManufacturerID", deserialize_with = "deserialize_hex")]
    pub manufacturer_id: u64,
    #[serde(rename = "@DeviceModelID", deserialize_with = "deserialize_hex")]
    pub device_model_id: u64,

    #[serde(rename = "SoftwareVersionID", default)]
    pub software_version_id: Vec<RDMSoftwareVersionID>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct RDMSoftwareVersionID {
    #[serde(rename = "@Value", deserialize_with = "deserialize_hex")]
    pub value: u64,

    #[serde(rename = "DMXPersonality", default)]
    pub dmx_personality: Vec<RDMDMXPersonality>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct RDMDMXPersonality {
    #[serde(rename = "@Value", deserialize_with = "deserialize_hex")]
    pub value: u64,
    #[serde(rename = "@DMXMode")]
    pub dmx_mode: NodePath<DMXMode>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct ArtNetProtocol {
    #[serde(rename = "Map")]
    maps: Vec<ProtocolMap>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct ProtocolMap {
    #[serde(rename = "@Key")]
    pub key: Uint,
    #[serde(rename = "@Value")]
    pub value: Uint,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct SACNProtocol {
    #[serde(rename = "Map")]
    maps: Vec<ProtocolMap>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct PosiStageNetProtocol {
    // Unused, but I wanted to parse it anyway
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct OpenSoundControlProtocol {
    // Unused, but I wanted to parse it anyway
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct CITPProtocol {
    // Unused, but I wanted to parse it anyway
}