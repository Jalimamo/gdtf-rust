use zip::ZipArchive;
use std::io::Cursor;
use serde::Deserialize;
use gdtf_derive::GdtfVerify;
use crate::parsing_helpers::{
    BeamType, DMXAddress, Float, FuseRating, Int, LampType, LaserColorType, Matrix, SemiUniqueBreak,
    StructureCrossSectionType, StructureType, SupportType, Vector3, WiringObjectComponentType, WiringOrientation
};
use crate::parsing_helpers::defaults::{
    default_color_temperature, default_luminous_flux, default_power_consumption, default_beam_radius, default_position,
    default_color_rendering_index
};
use crate::parsing_helpers::names_and_references::{
    Name, GeometryName, ModelName, OptionalNodePath, TopLevelGeometryName
};
use crate::parsing_helpers::resources::{Resource, TextureFile};
use crate::parsing_helpers::verification::{InitAtVerification, Verify, VerifySelf};
use crate::gdtf_xml_scheme::physical_descriptions::Emitter;

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
#[verify(verify_self = after_all)]
pub struct Geometries {
    #[serde(rename = "$value", default)]
    pub geometries: Vec<GeometryItem>
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(rename_all = "PascalCase")]
pub enum GeometryItem {
    Geometry(GeneralGeometry),
    Axis(AxisGeometry),
    FilterBeam(FilterBeamGeometry),
    FilterColor(FilterColorGeometry),
    FilterGobo(FilterGoboGeometry),
    FilterShaper(FilterShaperGeometry),
    Beam(BeamGeometry),
    MediaServerLayer(MediaServerLayerGeometry),
    MediaServerCamera(MediaServerCameraGeometry),
    MediaServerMaster(MediaServerMasterGeometry),
    Display(DisplayGeometry),
    GeometryReference(GeometryReference),
    Laser(LaserGeometry),
    WiringObject(WiringObjectGeometry),
    Inventory(InventoryGeometry),
    Structure(StructureGeometry),
    Support(SupportGeometry),
    Magnet(MagnetGeometry),
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct GeneralGeometry {
    #[serde(rename = "@Name")]
    pub name: Name,
    #[serde(rename = "@Model", default)]
    pub model: Option<ModelName>,
    #[serde(rename = "@Position", default = "default_position")]
    pub position: Matrix,

    #[serde(rename = "$value", default)]
    pub children: Vec<GeometryItem>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct AxisGeometry {
    #[serde(rename = "@Name")]
    pub name: Name,
    #[serde(rename = "@Model")]
    pub model: Option<ModelName>,
    #[serde(rename = "@Position", default = "default_position")]
    pub position: Matrix,

    #[serde(rename = "$value", default)]
    pub children: Vec<GeometryItem>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct FilterBeamGeometry {
    #[serde(rename = "@Name")]
    pub name: Name,
    #[serde(rename = "@Model", default)]
    pub model: Option<ModelName>,
    #[serde(rename = "@Position", default = "default_position")]
    pub position: Matrix,

    #[serde(rename = "$value", default)]
    pub children: Vec<GeometryItem>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct FilterColorGeometry {
    #[serde(rename = "@Name")]
    pub name: Name,
    #[serde(rename = "@Model", default)]
    pub model: Option<ModelName>,
    #[serde(rename = "@Position", default = "default_position")]
    pub position: Matrix,

    #[serde(rename = "$value", default)]
    pub children: Vec<GeometryItem>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct FilterGoboGeometry {
    #[serde(rename = "@Name")]
    pub name: Name,
    #[serde(rename = "@Model", default)]
    pub model: Option<ModelName>,
    #[serde(rename = "@Position", default = "default_position")]
    pub position: Matrix,

    #[serde(rename = "$value", default)]
    pub children: Vec<GeometryItem>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct FilterShaperGeometry {
    #[serde(rename = "@Name")]
    pub name: Name,
    #[serde(rename = "@Model", default)]
    pub model: Option<ModelName>,
    #[serde(rename = "@Position", default = "default_position")]
    pub position: Matrix,

    #[serde(rename = "$value", default)]
    pub children: Vec<GeometryItem>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
#[verify(verify_self)]
pub struct BeamGeometry {
    #[serde(rename = "@Name")]
    pub name: Name,
    #[serde(rename = "@Model")]
    pub model: Option<ModelName>,
    #[serde(rename = "@Position", default = "default_position")]
    pub position: Matrix,

    #[serde(rename = "@LampType", default)]
    pub lamp_type: LampType,
    #[serde(rename = "@PowerConsumption", default = "default_power_consumption")]
    pub power_consumption: Float,
    #[serde(rename = "@LuminousFlux", default = "default_luminous_flux")]
    pub luminous_flux: Float,
    #[serde(rename = "@ColorTemperature", default = "default_color_temperature")]
    pub color_temperature: Float,
    #[serde(rename = "@BeamAngle")]
    pub beam_angle: InitAtVerification<Float>,
    #[serde(rename = "@FieldAngle")]
    pub field_angle: InitAtVerification<Float>,
    #[serde(rename = "@ThrowRatio")]
    pub throw_ratio: InitAtVerification<Float>,
    #[serde(rename = "@RectangleRatio")]
    pub rectangle_ratio: InitAtVerification<Float>,
    #[serde(rename = "@BeamRadius", default = "default_beam_radius")]
    pub beam_radius: Float,
    #[serde(rename = "@BeamType", default)]
    pub beam_type: BeamType,
    #[serde(rename = "@ColorRenderingIndex", default = "default_color_rendering_index")]
    pub color_rendering_index: u8,
    #[serde(rename = "@EmitterSpectrum", default)]
    pub emitter_spectrum: OptionalNodePath<Emitter>,

    #[serde(rename = "$value", default)]
    pub children: Vec<GeometryItem>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct MediaServerLayerGeometry {
    #[serde(rename = "@Name")]
    pub name: Name,
    #[serde(rename = "@Model", default)]
    pub model: Option<ModelName>,
    #[serde(rename = "@Position", default = "default_position")]
    pub position: Matrix,

    #[serde(rename = "$value", default)]
    pub children: Vec<GeometryItem>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct MediaServerCameraGeometry {
    #[serde(rename = "@Name")]
    pub name: Name,
    #[serde(rename = "@Model", default)]
    pub model: Option<ModelName>,
    #[serde(rename = "@Position", default = "default_position")]
    pub position: Matrix,

    #[serde(rename = "$value", default)]
    pub children: Vec<GeometryItem>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct MediaServerMasterGeometry {
    #[serde(rename = "@Name")]
    pub name: Name,
    #[serde(rename = "@Model", default)]
    pub model: Option<ModelName>,
    #[serde(rename = "@Position", default = "default_position")]
    pub position: Matrix,

    #[serde(rename = "$value", default)]
    pub children: Vec<GeometryItem>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct DisplayGeometry {
    #[serde(rename = "@Name")]
    pub name: Name,
    #[serde(rename = "@Model", default)]
    pub model: Option<ModelName>,
    #[serde(rename = "@Position", default = "default_position")]
    pub position: Matrix,

    //TODO I have no clue if i implemented this right, because the Norm is not that specific in this case
    #[serde(rename = "@Texture")]
    pub texture: Resource<TextureFile>,

    #[serde(rename = "$value", default)]
    pub children: Vec<GeometryItem>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
#[verify(verify_self)]
pub struct LaserGeometry {
    #[serde(rename = "@Name")]
    pub name: Name,
    #[serde(rename = "@Model", default)]
    pub model: Option<ModelName>,
    #[serde(rename = "@Position", default = "default_position" )]
    pub position: Matrix,

    #[serde(rename = "@ColorType", default)]
    pub color_type: LaserColorType,
    #[serde(rename = "@Color")]
    pub color: InitAtVerification<Float>,
    //Deviation Some fixtures think they don't need output_strength
    #[serde(rename = "OutputStrength", default)]
    pub output_strength: Option<Float>,
    #[serde(rename = "Emitter", default)]
    pub emitter: OptionalNodePath<Emitter>,
    #[serde(rename = "BeamDiameter")]
    pub beam_diameter: Float,
    #[serde(rename = "BeamDivergenceMin")]
    pub beam_divergence_min: Float,
    #[serde(rename = "BeamDivergenceMax")]
    pub beam_divergence_max: Float,
    #[serde(rename = "ScanAnglePan")]
    pub scan_angle_pan: Float,
    #[serde(rename = "ScanAngleTilt")]
    pub scan_angle_tilt: Float,
    #[serde(rename = "ScanSpeed")]
    pub scan_speed: Float,

    #[serde(rename = "$value", default)]
    pub children: Vec<LaserChildren>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(rename_all = "PascalCase")]
pub enum LaserChildren {
    Protocol(ProtocolNode),

    #[serde(untagged)]
    Geometry(GeometryItem)
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct ProtocolNode {
    #[serde(rename = "@Name")]
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
#[verify(verify_self)]
pub struct GeometryReference {
    #[serde(rename = "@Name")]
    pub name: Name,
    #[serde(rename = "@Position", default = "default_position")]
    pub position: Matrix,
    #[serde(rename = "@Geometry")]
    pub geometry: TopLevelGeometryName,
    #[serde(rename = "@Model", default)]
    pub model: Option<ModelName>,

    #[serde(rename = "Break", default)]
    pub dmx_break: Vec<GeometryReferenceDmxBreak>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct GeometryReferenceDmxBreak {
    #[serde(rename = "@DMXOffset")]
    pub dmx_offset: DMXAddress,
    #[serde(rename = "@DMXBreak")]
    pub dmx_break: SemiUniqueBreak,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[verify(verify_self)]
pub struct WiringObjectGeometry {
    #[serde(rename = "@Name")]
    pub name: Name,
    #[serde(rename = "@Model", default)]
    pub model: Option<ModelName>,
    #[serde(rename = "@Position", default = "default_position")]
    pub position: Matrix,

    //Should only be use when component_type is not Fuse
    #[serde(rename = "ConnectorType")]
    pub connector_type: Option<Name>,

    #[serde(rename = "@ComponentType")]
    pub component_type: WiringObjectComponentType,

    //Should only be use when component_type is Consumer
    #[serde(rename = "@ElectricalPayLoad", default)]
    pub(crate) electrical_payload: InitAtVerification<Float>,
    #[serde(rename = "@VoltageRangeMax", default)]
    pub(crate) voltage_range_max: InitAtVerification<Float>,
    #[serde(rename = "@VoltageRangeMin", default)]
    pub(crate) voltage_range_min: InitAtVerification<Float>,
    #[serde(rename = "@FrequencyRangeMax", default)]
    pub(crate) frequency_range_max: InitAtVerification<Float>,
    #[serde(rename = "@FrequencyRangeMin", default)]
    pub(crate) frequency_range_min: InitAtVerification<Float>,
    #[serde(rename = "@CosPhi", default)]
    pub(crate) cos_phi: InitAtVerification<Float>,

    //Should only be use when component_type is PowerSource
    #[serde(rename = "@MaxPayload", default)]
    pub(crate) max_payload: InitAtVerification<Float>,
    #[serde(rename = "@Voltage", default)]
    pub(crate) voltage: InitAtVerification<Float>,

    // Should only be used when component_type is fuse
    #[serde(rename = "@FuseCurrent", default)]
    pub(crate) fuse_current: InitAtVerification<Float>,
    #[serde(rename = "@FuseRating", default)]
    pub(crate) fuse_rating: InitAtVerification<FuseRating>,

    // Can be used for alle component_types
    #[serde(rename = "@SignalType")]
    pub signal_type: String,
    #[serde(rename = "@PinCount")]
    pub pin_count: Int,
    #[serde(rename = "@SignalLayer")]
    pub signal_layer: Int,
    #[serde(rename = "@Orientation")]
    pub orientation: WiringOrientation,
    #[serde(rename = "@WireGroup")]
    pub wire_group: String,

    #[serde(rename = "$value", default)]
    pub children: Vec<WiringObjectChild>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(rename_all = "PascalCase")]
pub enum WiringObjectChild {
    PinPatch(PinPatchNode),

    #[serde(untagged)]
    Geometry(GeometryItem)
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct PinPatchNode {
    #[serde(rename = "@ToWiringObject")]
    pub to_wiring_object: GeometryName,
    #[serde(rename = "@FromPin")]
    pub(crate) from_pin: Int,
    #[serde(rename = "@ToPin")]
    pub to_pin: Int,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
#[verify(verify_self)]
pub struct InventoryGeometry {
    #[serde(rename = "@Name")]
    pub name: Name,
    #[serde(rename = "@Model", default)]
    pub model: Option<ModelName>,
    #[serde(rename = "@Position", default = "default_position")]
    pub position: Matrix,

    #[serde(rename = "@Count", default)]
    pub count: Int,

    #[serde(rename = "$value", default)]
    pub children: Vec<GeometryItem>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct StructureGeometry {
    #[serde(rename = "@Name")]
    pub name: Name,
    #[serde(rename = "@Model", default)]
    pub model: Option<ModelName>,
    #[serde(rename = "@Position", default = "default_position")]
    pub position: Matrix,

    #[serde(rename = "@LinkedGeometry")]
    pub linked_geometry: GeometryName,
    #[serde(rename = "@StructureType")]
    pub structure_type: StructureType,
    #[serde(rename = "@CrossSectionType ")]
    pub cross_section_type: StructureCrossSectionType,

    // Only use when cross_section_type is truss
    #[serde(rename = "@TrussCrossSection", default)]
    pub(crate) truss_cross_section: InitAtVerification<String>,

    // Only use when cross_section_type is tube
    #[serde(rename = "@CrossSectionHeight", default)]
    pub(crate) cross_section_height: InitAtVerification<Float>,
    #[serde(rename = "@CrossSectionWallThickness", default)]
    pub(crate) cross_section_wall_thickness: InitAtVerification<Float>,

    #[serde(rename = "$value", default)]
    pub children: Vec<GeometryItem>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
#[verify(verify_self)]
pub struct SupportGeometry {
    #[serde(rename = "@Name")]
    pub name: Name,
    #[serde(rename = "@Model", default)]
    pub model: Option<ModelName>,
    #[serde(rename = "@Position", default = "default_position")]
    pub position: Matrix,

    #[serde(rename = "@SupportType")]
    pub support_type: SupportType,
    #[serde(rename = "@RopeCrossSection")]
    pub(crate) rope_cross_section: InitAtVerification<String>,
    #[serde(rename = "@RopeOffset")]
    pub(crate) rope_offset: InitAtVerification<Vector3>,
    #[serde(rename = "@ResistanceX")]
    pub(crate) resistance_x: InitAtVerification<Float>,
    #[serde(rename = "@ResistanceY")]
    pub(crate) resistance_y: InitAtVerification<Float>,
    #[serde(rename = "@ResistanceZ")]
    pub(crate) resistance_z: InitAtVerification<Float>,
    #[serde(rename = "@ResistanceXX")]
    pub(crate) resistance_xx: InitAtVerification<Float>,
    #[serde(rename = "@ResistanceYY")]
    pub(crate) resistance_yy: InitAtVerification<Float>,
    #[serde(rename = "@ResistanceZZ")]
    pub(crate) resistance_zz: InitAtVerification<Float>,

    #[serde(rename = "@CapacityX")]
    pub capacity_x: Float,
    #[serde(rename = "@CapacityY")]
    pub capacity_y: Float,
    #[serde(rename = "@CapacityZ")]
    pub capacity_z: Float,
    #[serde(rename = "@CapacityXX")]
    pub capacity_xx: Float,
    #[serde(rename = "@CapacityYY")]
    pub capacity_yy: Float,
    #[serde(rename = "@CapacityZZ")]
    pub capacity_zz: Float,

    #[serde(rename = "$value", default)]
    pub children: Vec<GeometryItem>,
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct MagnetGeometry {
    #[serde(rename = "@Name")]
    pub name: Name,
    #[serde(rename = "@Model", default)]
    pub model: Option<ModelName>,
    #[serde(rename = "@Position", default = "default_position")]
    pub position: Matrix,

    #[serde(rename = "$value", default)]
    pub children: Vec<GeometryItem>,
}