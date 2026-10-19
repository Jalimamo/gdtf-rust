use crate::gdtf_xml_scheme::attribute_definitions::{Attribute, SubPhysicalUnit};
use crate::gdtf_xml_scheme::physical_descriptions::{ColorSpace, DmxProfile, Emitter, Filter, Gamut};
use crate::gdtf_xml_scheme::wheels::Wheel;
use crate::parsing_helpers::defaults::{
    default_channel_function_dmx_from, default_channel_function_physical_to, default_channel_function_real_acceleration,
    default_channel_function_real_fader, default_channel_set_from, default_dmx_break, default_dmx_change_time_limit,
    default_dmx_offset, default_ft_macro_step_duration, default_highlight_value, default_mib_fade, default_snap,
};
use crate::parsing_helpers::names_and_references::{
    GeometryName, ModeMasterNodePath, Name, Named, NodePath, OptionalNodePath, TopLevelGeometryName,
};
use crate::parsing_helpers::verification::{InitAtVerification, Verify, VerifySelf};
use crate::parsing_helpers::{
    DMXBreak, DMXOffset, DMXRelationType, DmxValue, Float, HighlightValue, Int, RespondToMaster, empty_name_as_none,
    empty_string_as_none, yes_no_bool,
};
use gdtf_derive::GdtfVerify;
use serde::Deserialize;
use std::io::Cursor;
use zip::ZipArchive;

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
#[verify(uniqueness = modes)]
pub struct DMXModes {
    #[serde(rename = "DMXMode", default)]
    pub modes: Vec<DMXMode>
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
#[verify(verify_self, push_to_roots)]
pub struct DMXMode {
    #[serde(rename = "@Name")]
    pub name: Name,
    #[serde(rename = "@Description", default, deserialize_with = "empty_string_as_none")]
    pub description: Option<String>,
    #[serde(rename = "@Geometry")]
    pub geometry: TopLevelGeometryName,

    #[serde(rename = "DMXChannels")]
    pub dmx_channels: DMXChannels,
    #[serde(rename = "Relations", default)]
    pub relations: Option<DMXRelations>,
    #[serde(rename = "FTMacros", default)]
    pub ft_macros: Option<FTMacros>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct DMXChannels {
    //Deviation: FUCK ACME FUCK ACME FUCK ACME FUCK ACME!!!!!
    #[serde(rename = "DMXChannel", default)]
    pub dmx_channels: Vec<DMXChannel>
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct DMXChannel {
    #[serde(rename = "@DMXBreak", default = "default_dmx_break")]
    pub dmx_break: DMXBreak,
    #[serde(rename = "@Offset", default = "default_dmx_offset")]
    pub offset: DMXOffset,
    #[serde(rename = "@InitialFunction")]
    pub initial_function: InitAtVerification<NodePath<ChannelFunction>>,
    #[serde(rename = "@Highlight", default = "default_highlight_value")]
    pub highlight: HighlightValue,
    #[serde(rename = "@Geometry")]
    pub geometry: GeometryName,

    #[serde(rename = "LogicalChannel", default)]
    pub logical_channels: Vec<LogicalChannel>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct LogicalChannel {
    #[serde(rename = "@Attribute")]
    pub attribute: NodePath<Attribute>,
    #[serde(rename = "@Snap", default = "default_snap", deserialize_with = "yes_no_bool")]
    pub snap: bool,
    #[serde(rename = "@Master", default)]
    pub respond_to_master: RespondToMaster,
    #[serde(rename = "@MibFade", default = "default_mib_fade")]
    pub mib_fade: Float,
    #[serde(rename = "@DMXChangeTimeLimit", default = "default_dmx_change_time_limit")]
    pub dmx_change_time_limit: Float,

    #[serde(rename = "ChannelFunction", default)]
    pub channel_functions: Vec<ChannelFunction>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[verify(push_to_roots)]
pub struct ChannelFunction {
    #[serde(rename = "@Name")]
    pub name: Name,
    // When this is None, it should count as NoFeature
    #[serde(rename = "@Attribute")]
    pub attribute: OptionalNodePath<Attribute>,
    #[serde(rename = "@OriginalAttribute", default,  deserialize_with = "empty_string_as_none")]
    pub original: Option<String>,
    #[serde(rename = "@DMXFrom", default = "default_channel_function_dmx_from")]
    pub dmx_from: DmxValue,
    #[serde(rename = "@Default")]
    pub default: InitAtVerification<DmxValue>,
    #[serde(rename = "@PhysicalFrom", default = "default_channel_function_physical_to")]
    pub physical_from: Float,
    #[serde(rename = "@PhysicalTo", default = "default_channel_function_physical_to")]
    pub physical_to: Float,
    #[serde(rename = "@RealFade", default = "default_channel_function_real_fader")]
    pub real_fade: Float,
    #[serde(rename = "@RealAcceleration", default = "default_channel_function_real_acceleration")]
    pub real_acceleration: Float,
    #[serde(rename = "@Wheel", default)]
    pub wheel: OptionalNodePath<Wheel>,
    #[serde(rename = "@Emitter", default)]
    pub emitter: OptionalNodePath<Emitter>,
    #[serde(rename = "@Filter", default)]
    pub filter: OptionalNodePath<Filter>,
    #[serde(rename = "@ColorSpace", default)]
    pub color_space: OptionalNodePath<ColorSpace>,
    #[serde(rename = "@Gamut", default)]
    pub gamut: OptionalNodePath<Gamut>,

    #[serde(rename = "@ModeMaster", default)]
    mode_master: ModeMasterNodePath,
    #[serde(rename = "@ModeFrom", default)]//TODO = "default_mode_master_from")]
    mode_from: InitAtVerification<DmxValue>,
    #[serde(rename = "@ModeTo", default)]//TODO = "default_mode_master_to")]
    mode_to: InitAtVerification<DmxValue>,



    #[serde(rename = "@DMXProfile")]
    dmx_profile: OptionalNodePath<DmxProfile>,
    //The two following Attributes should be only used when dmx_profile is Some(..)
    #[serde(rename = "@Min", default)]
    dmx_profile_min: InitAtVerification<Float>,
    #[serde(rename = "@Max", default)]
    dmx_profile_max: InitAtVerification<Float>,

    #[serde(rename = "@CustomName", default)]
    pub custom_name: InitAtVerification<String>,

    #[serde(rename = "ChannelSet", default)]
    pub channel_sets: Vec<ChannelSet>,
    #[serde(rename = "SubChannelSet", default)]
    pub sub_channel_sets: Vec<SubChannelSet>
}


#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct ChannelSet {
    #[serde(rename = "@Name", default, deserialize_with = "empty_name_as_none")]
    pub name: Option<Name>,
    #[serde(rename = "@DMXFrom", default = "default_channel_set_from")]
    pub dmx_from: DmxValue,
    #[serde(rename = "@PhysicalFrom", default)]
    pub physical_from: InitAtVerification<Float>,
    #[serde(rename = "@PhysicalTo", default)]
    pub physical_to: InitAtVerification<Float>,
    #[serde(rename = "@WheelSlotIndex", default)]
    pub wheel_slot_index: Option<Int>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct SubChannelSet {
    #[serde(rename = "@Name", default, deserialize_with = "empty_name_as_none")]
    pub name: Option<Name>,
    #[serde(rename = "@PhysicalFrom", default)]
    pub physical_from: Option<Float>,
    #[serde(rename = "@PhysicalTo", default)]
    pub physical_to: Option<Float>,
    #[serde(rename = "@SubPhysicalUnit")]
    pub sub_physical_unit: NodePath<SubPhysicalUnit>,
    #[serde(rename = "@DMXProfile", default)]
    pub dmx_profile: OptionalNodePath<DmxProfile>,
}


#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct DMXRelations {
    #[serde(rename = "Relation", default)]
    pub dmx_channels: Vec<DMXRelation>
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct DMXRelation {
    #[serde(rename = "@Name")]
    pub name: Name,
    #[serde(rename = "@Master")]
    pub master: NodePath<DMXChannel>,
    #[serde(rename = "@Follower")]
    pub follower: NodePath<ChannelFunction>,
    #[serde(rename = "@Type")]
    pub relation_type: DMXRelationType,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct FTMacros {
    #[serde(rename = "FTMacro", default)]
    pub ft_macros: Vec<FTMacro>
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct FTMacro {
    #[serde(rename = "@Name")]
    pub name: Name,
    #[serde(rename = "@ChannelFunction", default)]
    pub channel_function: OptionalNodePath<ChannelFunction>,

    #[serde(rename = "MacroDMX", default)]
    pub macro_dmx: Option<FTMacroDMX>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct FTMacroDMX {
    #[serde(rename = "MacroDMXStep", default)]
    pub steps: Vec<FTMacroDMXStep>
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct FTMacroDMXStep {
    #[serde(rename = "@Duration", default = "default_ft_macro_step_duration")]
    pub duration: Float,
    #[serde(rename = "MacroDMXValue", default)]
    pub macro_dmx_value: Vec<FTMacroDMXValue>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct FTMacroDMXValue {
    #[serde(rename = "@Value")]
    pub value: DmxValue,
    #[serde(rename = "@DMXChannel")]
    pub channel: NodePath<DMXChannel>,
}