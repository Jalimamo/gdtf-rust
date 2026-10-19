use std::any::{Any, TypeId};
use std::cmp::PartialEq;
use std::collections::{HashMap, HashSet};
use std::io::{Cursor};
use bytes::Bytes;
use serde::{Deserialize, Deserializer};
use zip::ZipArchive;
use crate::gdtf_xml_scheme::dmx_modes::{DMXMode, LogicalChannel};
use crate::parsing_helpers::{Int, Uint, Float, DMXOffset, GdtfDate, ColorCie, Resource, ResourceType, BeamType, LaserColorType, WiringObjectComponentType, StructureCrossSectionType, SupportType};
use crate::gdtf_xml_scheme::FixtureType;
use crate::gdtf_xml_scheme::geometries::{BeamGeometry, Geometries, GeometryItem, GeometryReference, InventoryGeometry, LaserGeometry, StructureGeometry, SupportGeometry, WiringObjectChild, WiringObjectGeometry};
use crate::gdtf_xml_scheme::physical_descriptions::{Emitter, Filter, PhysicalDescriptions};
use crate::parsing_helpers::defaults::{
    default_beam_angle, default_field_angle, default_rectangle_ratio, default_throw_ratio
};
use crate::parsing_helpers::names_and_references::{GeometryName, ModeMasterNodePath, ModelName, Name, Named, NodePath, OptionalNodePath, ResolveNodePath, TopLevelGeometryName};
use crate::parsing_helpers::names_and_references::PathResult::*;
use crate::parsing_helpers::resources::ResourceError::*;
use crate::parsing_helpers::verification::InitAtVerification::*;

#[derive(Default)]
pub struct VerificationRoots {
    roots: HashMap<TypeId, Box<dyn Verify>>,
}

impl VerificationRoots {
    pub fn new() -> Self {
        Self { roots: HashMap::new() }
    }

    pub fn push<T>(&mut self, root: Box<T>) where T: Verify + 'static {
        self.roots.insert(TypeId::of::<T>(), root);
    }

    pub fn pop<T>(&mut self) where T: Verify + 'static {
        self.roots.remove(&TypeId::of::<T>());
    }

    pub fn get_mut_from_type<T: 'static>(&mut self) -> Option<&mut T> {
        self.roots
            .get_mut(&TypeId::of::<T>())
            .and_then(|root|  root.as_any_mut().downcast_mut::<T>())
    }
}

pub trait Verify {
    fn verify(&mut self, _roots: &mut VerificationRoots, _archive: &mut ZipArchive<Cursor<Bytes>>)
        -> Result<Vec<String>, String>
    {Ok(Vec::new())}

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait VerifySelf:Verify {
    fn verify_self(&mut self, _roots: &mut VerificationRoots) -> Result<Vec<String>, String> {Ok(Vec::new())}
}

fn verify_items<'a, I>(items: I, roots: &mut VerificationRoots, archive: &mut ZipArchive<Cursor<Bytes>>)
    -> Result<Vec<String>, String> where I: IntoIterator<Item=&'a mut dyn Verify>
{
    let mut warnings = Vec::new();

    for item in items {
        warnings.extend(item.verify(roots, archive)?);
    }

    Ok(warnings)
}

impl<T: Verify + 'static> Verify for Vec<T> {
    fn verify(&mut self, roots: &mut VerificationRoots, archive: &mut ZipArchive<Cursor<Bytes>>)
        -> Result<Vec<String>, String>
    {
        verify_items(self.iter_mut().map(|i| i as &mut dyn Verify), roots, archive)
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {self}
}

impl<T: Verify + 'static, const N: usize> Verify for [T; N] {
    fn verify(&mut self, roots: &mut VerificationRoots, archive: &mut ZipArchive<Cursor<Bytes>>)
        -> Result<Vec<String>, String>
    {
        verify_items(self.iter_mut().map(|i| i as &mut dyn Verify), roots, archive)
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {self}
}

impl<T: Verify + 'static> Verify for Option<T> {
    fn verify(&mut self, roots: &mut VerificationRoots, archive: &mut ZipArchive<Cursor<Bytes>>)
        -> Result<Vec<String>, String>
    {
        match self {
            Some(value) => value.verify(roots, archive),
            None => Ok(Vec::new()),
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {self}
}




impl Verify for String {fn as_any_mut(&mut self) -> &mut dyn Any {self}}
impl Verify for bool {fn as_any_mut(&mut self) -> &mut dyn Any {self}}
impl Verify for Float {fn as_any_mut(&mut self) -> &mut dyn Any {self}}
impl Verify for Int {fn as_any_mut(&mut self) -> &mut dyn Any {self}}
impl Verify for Uint {fn as_any_mut(&mut self) -> &mut dyn Any {self}}
impl Verify for u32 {fn as_any_mut(&mut self) -> &mut dyn Any {self}}
impl Verify for u16 {fn as_any_mut(&mut self) -> &mut dyn Any {self}}
impl Verify for u8 {fn as_any_mut(&mut self) -> &mut dyn Any {self}}
impl Verify for f32 {fn as_any_mut(&mut self) -> &mut dyn Any {self}}
impl Verify for DMXOffset {fn as_any_mut(&mut self) -> &mut dyn Any {self}}
impl Verify for GdtfDate {fn as_any_mut(&mut self) -> &mut dyn Any {self}}


impl Verify for ColorCie {
    fn verify(&mut self, _roots: &mut VerificationRoots, _archive: &mut ZipArchive<Cursor<Bytes>>)
        -> Result<Vec<String>, String>
    {
        let mut warnings = Vec::new();

        if self.x < 0.0 || self.x > 1.0 {
            return Err(format!("CIExyY Error: x out of Range [0,1]. Got {}.", self.x))
        } else if self.y < 0.0 || self.y > 1.0 {
            return Err(format!("CIExyY Error: y out of Range [0,1]. Got {}.", self.y))
        } else if self.yy < 0.0 || self.yy > 100.0 {
            warnings.push(format!(
                "CIExyY Warning: Y out of Range [0,100]. Got {}. Clamped.",
                self.yy
            ));

            self.yy = self.yy.clamp(0.0, 100.0);
        }
        Ok(warnings)
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {self}
}

impl<T: ResourceType + 'static> Verify for Resource<T> {
    fn verify(&mut self, _roots: &mut VerificationRoots, archive: &mut ZipArchive<Cursor<Bytes>>)
        -> Result<Vec<String>, String>
    {
        match self.get_file(archive) {
            Err(UnzipError(e)) => Err(format!("Unzip error: {}", e)),
            Err(NoFileFoundError) => Ok(vec![format!("File {} not found. Using default", self.0)]),
            Ok(_) => Ok(Vec::new())
        }
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {self}
}

impl<T: Verify + 'static> Verify for NodePath<T> where FixtureType: ResolveNodePath<T> {

    fn verify(&mut self, roots: &mut VerificationRoots, _archive: &mut ZipArchive<Cursor<Bytes>>)
        -> Result<Vec<String>, String>
    {
        match self.get_from_roots(roots) {
            Exists(_) => Ok(Vec::new()),
            PathWrong => Err(format!("Invalid Node Path format: {}", self.0.join("."))),
            Missing => Err(format!("Unresolved NodePath: Target '{}' does not exist", self.0.join("."))),
            InvalidRoot => Err(format!("NodePath validation failed: GDTF root document context is missing: {}", self.0.join("."))),
        }
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {self}
}

impl<T: Verify + 'static> Verify for OptionalNodePath<T> where FixtureType: ResolveNodePath<T> {
    fn verify(&mut self, roots: &mut VerificationRoots, archive: &mut ZipArchive<Cursor<Bytes>>)
        -> Result<Vec<String>, String>
    {
        let mut warnings = Vec::new();

        if let Some(path) = &mut self.0 {
            match path.verify(roots, archive) {
                Ok(path_warnings) => {
                    warnings.extend(path_warnings);
                }
                Err(e) => {
                    warnings.push(format!(
                        "Optional NodePath reference failed validation and was removed. Original error: {}", e
                    ));

                    self.0 = None;
                }
            }
        }

        Ok(warnings)

    }

    fn as_any_mut(&mut self) -> &mut dyn Any {self}
}

impl Verify for ModeMasterNodePath {
    fn verify(&mut self, roots: &mut VerificationRoots, _archive: &mut ZipArchive<Cursor<Bytes>>)
              -> Result<Vec<String>, String>
    {
        match self.get(roots) {
            Exists(_) => Ok(Vec::new()),
            PathWrong => Err(format!("Invalid Node Path format: {}", self.path_string())),
            Missing => Err(format!("Unresolved NodePath: Target '{}' does not exist", self.path_string())),
            InvalidRoot => Err(format!(
                "NodePath validation failed: GDTF root document context is missing: {}", self.path_string()
            )),
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {self}
}

impl Verify for ModelName {
    fn verify(&mut self, roots: &mut VerificationRoots, archive: &mut ZipArchive<Cursor<Bytes>>)
              -> Result<Vec<String>, String>
    {
        let mut warnings = Vec::new();
        warnings.extend(self.0.verify(roots, archive)?);

        let model = roots.get_mut_from_type::<FixtureType>()
            .and_then(|fixture_type| fixture_type.models.as_ref())
            .and_then(|models| self.get_model(models));

        if model.is_some() {
            Ok(Vec::new())
        } else {
            Err(format!("Model {} not found", self))
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {self}
}
impl Verify for GeometryName {

    fn verify(&mut self, roots: &mut VerificationRoots, _archive: &mut ZipArchive<Cursor<Bytes>>)
              -> Result<Vec<String>, String>
    {
        let Some(geometries) = roots.get_mut_from_type::<FixtureType>()
            .map(|fixture_type| &fixture_type.geometries.geometries)
        else {
            unreachable!();
        };

        if self.get_this_geometry(geometries).is_none() {
            Err(format!("Geometry {} not found", self))
        } else {
            Ok(Vec::new())
        }


        // None of the above ifs should fail, if the gdtf has been parsed correctly
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {self}
}

impl Verify for TopLevelGeometryName {
    fn verify(&mut self, roots: &mut VerificationRoots, _archive: &mut ZipArchive<Cursor<Bytes>>)
              -> Result<Vec<String>, String>
    {
        let Some(geometries) = roots.get_mut_from_type::<FixtureType>()
            .map(|fixture_type| &fixture_type.geometries.geometries)
        else {
            unreachable!();
        };

        if self.get_this_geometry(geometries).is_some() {
            Ok(Vec::new())
        } else {
            Err(format!("Top-Level-Geometry {} not found", self))
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {self}
}
impl<T> Verify for InitAtVerification<T> where T: Verify + 'static {
    fn as_any_mut(&mut self) -> &mut dyn Any {self}
}







#[derive(Debug, Clone, Copy, Default)]
pub enum InitAtVerification<T> {
    Unverified(T),
    #[default]
    Unspecified,
    Verified(T),
    Unused,
}

impl<T> InitAtVerification<T> {

    fn is_unspecified(&self) -> bool {
        matches!(self, Unspecified)
    }

    fn discard_unverified(&mut self) -> bool {
        match self {
            Unverified(..) => {
                *self = Unused;
                true
            },
            _ => false
        }
    }

    fn make_verified(&mut self) {
        *self = match std::mem::replace(self, Unspecified) {
            Unverified(val) => Verified(val),
            Unspecified => Unused,
            other => other
        };
    }

    fn verify_or_default(&mut self, default: T) -> &T {
        match self {
            Unverified(val) => val,
            Verified(val) => val,
            Unspecified => {
                *self = Verified(default);

                match self {
                    Verified(val) => val,
                    _ => unreachable!(),
                }
            }
            Unused => {
                *self = Verified(default);

                match self {
                    Verified(val) => val,
                    _ => unreachable!(),
                }
            }
        }
    }
}

impl<'de, T> Deserialize<'de> for InitAtVerification<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<T>::deserialize(deserializer)?;
        match opt {
            Some(x) => Ok(Unverified(x)),
            None => Ok(Unspecified),
        }
    }
}


impl VerifySelf for Emitter {
    fn verify_self(&mut self, _roots: &mut VerificationRoots) -> Result<Vec<String>, String> {
        let mut warnings = Vec::new();

        if self.color.is_none() && self.dominant_wave_length.is_none() {
            warnings.push(format!(
                "Emitter '{}' is missing required 'DominantWaveLength' because 'Color' is omitted.",
                self.name
            ));
        }

        for measurement in &self.measurements {
            if measurement.luminous_intensity.is_none() {
                return Err(format!("Emitter {} has a measurement without luminous intensity", self.name))
            }
            if measurement.transmission.is_some() {
                warnings.push(format!(
                    "Emitter {} has a measurement with transmission, which will be ignored", self.name
                ))
            }
            if measurement.physical_unit.0.eq(&0.0) {
                warnings.push(format!(
                    "Emitter '{}' defines a measurement at physical percentage 0. This is redundant and contradicts \
                     DIN SPEC 15800, but processing will continue.", self.name
                ))
            }
        }

        Ok(warnings)
    }
}

impl VerifySelf for Filter {
    fn verify_self(&mut self, _roots: &mut VerificationRoots) -> Result<Vec<String>, String> {
        let mut warnings = Vec::new();

        for measurement in &self.measurements {
            if measurement.luminous_intensity.is_some() {
                warnings.push(format!(
                    "Filter {} has a measurement with luminous intensity, which will be ignored", self.name
                ))
            }
            if measurement.transmission.is_none() {
                return Err(format!("Filter {} has a measurement without transmission", self.name))
            }
        }

        Ok(warnings)
    }
}

impl VerifySelf for PhysicalDescriptions {
    fn verify_self(&mut self, _roots: &mut VerificationRoots) -> Result<Vec<String>, String> {
        let mut seen_names = HashSet::new();

        if let Some(main_color_space) = &self.color_space {
            seen_names.insert(main_color_space.name.as_str());
        }

        if let Some(additional) = &self.additional_color_spaces {
            for cs in &additional.color_spaces {
                if !seen_names.insert(cs.name.as_str()) {
                    return Err(format!(
                        "Duplicate ColorSpace name '{}' detected. Names must be unique across the default ColorSpace \
                         and AdditionalColorSpaces.", cs.name
                    ));
                }
            }
        }


        Ok(Vec::new())
    }
}

impl VerifySelf for BeamGeometry {
    fn verify_self(&mut self, _roots: &mut VerificationRoots) -> Result<Vec<String>, String> {
        let mut warnings = Vec::new();

        match self.beam_type {
            BeamType::Wash | BeamType::Spot | BeamType::PC | BeamType::Fresnel | BeamType::Glow => {
                let beam_angle = *self.beam_angle.verify_or_default(default_beam_angle());
                let field_angle = *self.field_angle.verify_or_default(default_field_angle());

                if beam_angle > field_angle {
                    warnings.push(format!(
                        "Beam '{}' defines a BeamAngle ({}) that is greater than its FieldAngle ({}). This is \
                        physically invalid.", self.name, beam_angle, field_angle
                    ))
                }

                if self.throw_ratio.discard_unverified() {
                    warnings.push(format!(
                        "Beam '{}' defines a ThrowRatio, which is only valid for BeamType 'Rectangle'. The value will \
                         be ignored and removed.", self.name
                    ));
                }

                if self.rectangle_ratio.discard_unverified() {
                    warnings.push(format!(
                        "Beam '{}' defines a RectangleRatio, which is only valid for BeamType 'Rectangle'. The value \
                         will be ignored and removed.", self.name
                    ));
                }
            }

            BeamType::Rectangle => {
                self.throw_ratio.verify_or_default(default_throw_ratio());
                self.rectangle_ratio.verify_or_default(default_rectangle_ratio());

                if self.beam_angle.discard_unverified() {
                    warnings.push(format!(
                        "Beam '{}' of type 'Rectangle' defines a BeamAngle. This property is ignored for rectangular \
                         beams.", self.name
                    ));
                }

                if self.field_angle.discard_unverified() {
                    warnings.push(format!(
                        "Beam '{}' of type 'Rectangle' defines a FieldAngle. This property is ignored for rectangular \
                         beams.", self.name
                    ));
                }

            }
            BeamType::None => {
                if self.beam_angle.discard_unverified() {
                    warnings.push(format!(
                        "Beam '{}' of type 'None' defines a BeamAngle, which will be ignored.", self.name
                    ));
                }
                if self.field_angle.discard_unverified() {
                    warnings.push(format!(
                        "Beam '{}' of type 'None' defines a FieldAngle, which will be ignored.", self.name
                    ));
                }
                if self.throw_ratio.discard_unverified() {
                    warnings.push(format!(
                        "Beam '{}' of type 'None' defines a ThrowRatio, which will be ignored.", self.name
                    ));
                }
                if self.rectangle_ratio.discard_unverified() {
                    warnings.push(format!(
                        "Beam '{}' of type 'None' defines a RectangleRatio, which will be ignored.", self.name
                    ));
                }
            }
        }

        Ok(warnings)
    }
}

impl VerifySelf for Geometries {
    // fn verify_self(&mut self, _roots: &mut VerificationRoots) -> Result<Vec<String>, String> {
    //     let mut seen = HashSet::new();
    //     collect_geometry_names(self.geometries.iter(), &mut seen)?;
    //     Ok(Vec::new())
    // }
    fn verify_self(&mut self, _roots: &mut VerificationRoots) -> Result<Vec<String>, String> {
        let _refs = TopLevelGeometryName(Name::new("Base").unwrap()).get_referenced_geometries(&*self.geometries);
        Ok(Vec::new())
    }
}

fn collect_geometry_names<'a>(
    geometries: impl Iterator<Item = &'a GeometryItem>, seen: &mut HashSet<String>
) -> Result<(), String>
{
    for geometry in geometries {
        let name = geometry.name();
        if !seen.insert(name.clone()) {
            return Err(format!("Duplicate geometry name '{}'", name));
        }
        collect_geometry_names(GeometryName::children(geometry), seen)?;
    }
    Ok(())
}

impl VerifySelf for LaserGeometry {
    fn verify_self(&mut self, _roots: &mut VerificationRoots) -> Result<Vec<String>, String> {
        let mut warnings = Vec::new();

        match self.color_type {
            LaserColorType::RGB => {
                if self.color.discard_unverified() {
                    warnings.push(format!(
                        "Laser '{}' defines a 'Color' value, but its ColorType is 'RGB'. The 'Color' attribute is \
                        only required for 'SingleWaveLength' and will be ignored.", self.name
                    ));
                }
            }
            LaserColorType::SingleWaveLength => {
                if self.color.is_unspecified() {
                    return Err(format!(
                        "Laser '{}' has ColorType 'SingleWaveLength' but is missing the required 'Color' attribute.",
                        self.name
                    ));
                } else {
                    self.color.make_verified();
                }
            }
        }

        let (min, max) = (self.beam_divergence_min, self.beam_divergence_max);
        if min > max {
            warnings.push(format!(
                "Laser '{}' defines a BeamDivergenceMin ({}) that is greater than its BeamDivergenceMax ({}). \
                 This is physically invalid.",self.name, min, max
            ))
        }

        Ok(warnings)
    }
}

impl VerifySelf for WiringObjectGeometry {
    fn verify_self(&mut self, roots: &mut VerificationRoots) -> Result<Vec<String>, String> {
        let mut warnings = Vec::new();

        if self.component_type != WiringObjectComponentType::Consumer {
            if self.electrical_payload.discard_unverified() {
                warnings.push(format!(
                    "WiringObject '{}' defines 'ElectricalPayload', but ComponentType is not 'Consumer'. \
                    Property will be ignored and removed.", self.name
                ));
            }
            if self.voltage_range_max.discard_unverified() | self.voltage_range_min.discard_unverified() {
                warnings.push(format!(
                    "WiringObject '{}' defines voltage range properties, but ComponentType is not 'Consumer'. \
                    Properties will be ignored and removed.", self.name
                ));
            }
            if self.frequency_range_max.discard_unverified() | self.frequency_range_min.discard_unverified() {
                warnings.push(format!(
                    "WiringObject '{}' defines frequency range properties, but ComponentType is not 'Consumer'. \
                    Properties will be ignored and removed.", self.name
                ));
            }
            if self.cos_phi.discard_unverified() {
                warnings.push(format!(
                    "WiringObject '{}' defines 'CosPhi', but ComponentType is not 'Consumer'. \
                    Property will be ignored and removed.", self.name
                ));
            }
        } else {
            self.electrical_payload.make_verified();
            self.voltage_range_min.make_verified();
            self.voltage_range_max.make_verified();
            self.frequency_range_min.make_verified();
            self.frequency_range_max.make_verified();
            if let (Verified(min), Verified(max)) = (self.voltage_range_min, self.voltage_range_max) {
                if min > max {
                    warnings.push(format!(
                        "WiringObject '{}' defines VoltageRangeMin ({}) greater than VoltageRangeMax ({}). This is \
                         physically invalid.", self.name, min, max
                    ));
                }
            }
            if let (Verified(min), Verified(max)) = (self.frequency_range_min, self.frequency_range_max) {
                if min > max {
                    warnings.push(format!(
                        "WiringObject '{}' defines FrequencyRangeMin ({}) greater than FrequencyRangeMax ({}). \
                         This is physically invalid.", self.name, min, max
                    ));
                }
            }
        }

        if self.component_type != WiringObjectComponentType::PowerSource {
            if self.max_payload.discard_unverified() {
                warnings.push(format!(
                    "WiringObject '{}' defines 'MaxPayload', but ComponentType is not 'PowerSource'. Property will be \
                     ignored and removed.", self.name
                ));
            }
            if self.voltage.discard_unverified() {
                warnings.push(format!(
                    "WiringObject '{}' defines 'Voltage', but ComponentType is not 'PowerSource'. Property will be \
                    ignored and removed.", self.name
                ));
            }
        } else {
            self.max_payload.make_verified();
            self.voltage.make_verified();
        }

        if self.component_type != WiringObjectComponentType::Fuse {
            if self.fuse_current.discard_unverified() {
                warnings.push(format!(
                    "WiringObject '{}' defines 'FuseCurrent', but ComponentType is not 'Fuse'. Property will be \
                    ignored and removed.", self.name
                ));
            }
            if self.fuse_rating.discard_unverified() {
                warnings.push(format!(
                    "WiringObject '{}' defines 'FuseRating', but ComponentType is not 'Fuse'. Property will be \
                    ignored and removed.", self.name
                ));
            }
        } else {
            self.fuse_current.make_verified();
            self.fuse_rating.make_verified();
        }

        let pin_count_this = self.pin_count;
        for child in &self.children {
            if let WiringObjectChild::PinPatch(patch) = child {
                if patch.from_pin > pin_count_this || patch.from_pin < 1 {
                    warnings.push(format!(
                        "WiringObject '{}' contains a PinPatch referencing FromPin '{}', which exceeds the allowed \
                        PinCount range (1-{}).", self.name, patch.from_pin, pin_count_this
                    ));
                }

                let Some(geometries) = roots.get_mut_from_type::<FixtureType>()
                    .map(|fixture_type| &fixture_type.geometries.geometries)
                else {
                    unreachable!();
                };

                let target_geometry = patch.to_wiring_object.get_this_geometry(
                    geometries
                );

                if let Some(GeometryItem::WiringObject(target)) = target_geometry {
                    let pin_count_other = target.pin_count;

                    if patch.to_pin > pin_count_other || patch.to_pin < 1 {
                        warnings.push(format!(
                            "WiringObject '{}' contains a PinPatch referencing ToPin '{}', which exceeds the \
                            allowed PinCount range (1-{}) of target '{}'.",
                            self.name, patch.to_pin, pin_count_other, patch.to_wiring_object
                        ));
                    } else {
                        warnings.push(format!(
                            "WiringObject '{}' contains a PinPatch referencing target WiringObject '{}', \
                            which could not be found in the geometry tree.", self.name, patch.to_wiring_object
                        ));
                    }
                }

            }
        }

        Ok(warnings)
    }
}

impl VerifySelf for InventoryGeometry {
    fn verify_self(&mut self, _roots: &mut VerificationRoots) -> Result<Vec<String>, String> {
        if self.count < 0 {
            Err(format!("WiringObject '{}' contains a count of less than 0. This isn't technically against the \
            Norm, but it doesnt make any sense. How do you have {} of something?", self.name, self.count))
        } else {
            Ok(Vec::new())
        }
    }
}

impl VerifySelf for StructureGeometry {
    fn verify_self(&mut self, _roots: &mut VerificationRoots) -> Result<Vec<String>, String> {
        let mut warnings = Vec::new();

        match self.cross_section_type {
            StructureCrossSectionType::Tube => {
                if self.truss_cross_section.discard_unverified() {
                    warnings.push(format!(
                        "Structure '{}' defines a 'TrussCrossSection', but its CrossSectionType is 'Tube'. The \
                        property is only valid for trusses and will be removed.", self.name
                    ));
                }
                self.cross_section_height.make_verified();
                self.cross_section_wall_thickness.make_verified();
            }
            StructureCrossSectionType::TrussFramework => {
                if self.cross_section_height.discard_unverified() {
                    warnings.push(format!(
                        "Structure '{}' defines a 'CrossSectionHeight', but its CrossSectionType is 'TrussFramework'. \
                         The property is only valid for tubes and will be removed.", self.name
                    ));
                }

                if self.cross_section_wall_thickness.discard_unverified() {
                    warnings.push(format!(
                        "Structure '{}' defines a 'CrossSectionWallThickness', but its CrossSectionType is \
                        'TrussFramework'. The property is only valid for tubes and will be removed.", self.name
                    ));
                }

                self.truss_cross_section.make_verified();
            }
        }

        Ok(warnings)
    }
}

impl VerifySelf for SupportGeometry {
    fn verify_self(&mut self, _roots: &mut VerificationRoots) -> Result<Vec<String>, String> {
        let mut warnings = Vec::new();

        match self.support_type {
            SupportType::GroundSupport => {
                // Rope-Eigenschaften bei GroundSupports bereinigen
                if self.rope_cross_section.discard_unverified() {
                    warnings.push(format!(
                        "Support '{}' defines a 'RopeCrossSection', but its SupportType is 'GroundSupport'. This \
                        property is only valid for ropes and will be removed.", self.name
                    ));
                }

                if self.rope_offset.discard_unverified() {
                    warnings.push(format!(
                        "Support '{}' defines a 'RopeOffset', but its SupportType is 'GroundSupport'. This property is \
                         only valid for ropes and will be removed.", self.name
                    ));
                }
                self.resistance_x.make_verified();
                self.resistance_y.make_verified();
                self.resistance_z.make_verified();
                self.resistance_xx.make_verified();
                self.resistance_yy.make_verified();
                self.resistance_zz.make_verified();
            }
            SupportType::Rope => {
                let has_resistance = self.resistance_x.discard_unverified() |
                    self.resistance_y.discard_unverified() |
                    self.resistance_z.discard_unverified() |
                    self.resistance_xx.discard_unverified() |
                    self.resistance_yy.discard_unverified() |
                    self.resistance_zz.discard_unverified();

                if has_resistance {
                    warnings.push(format!(
                        "Support '{}' defines one or more Resistance properties, but its SupportType is 'Rope'. These \
                        properties are only valid for GroundSupports and will be removed.", self.name
                    ));
                }

                self.rope_cross_section.make_verified();
                self.rope_offset.make_verified();
            }
        }

        Ok(warnings)
    }
}

impl VerifySelf for GeometryReference {
    fn verify_self(&mut self, _roots: &mut VerificationRoots) -> Result<Vec<String>, String> {
        let mut seen = HashSet::new();
        let mut repeated_once = false;
        for dmx_break in &mut self.dmx_break {
            let name = dmx_break.name();
            if !seen.insert(name.clone()) {
                if !repeated_once {
                    println!("before: {:?}", dmx_break.dmx_break);

                    dmx_break.dmx_break.to_overwrite();

                    println!("after: {:?}", dmx_break.dmx_break);
                    repeated_once = true;
                } else {
                    return Err(format!(
                        "DMXBreak '{}' is not unique within GeometryReference {}", name, self.name
                    ));
                }
            }
        }


        Ok(Vec::new())
    }
}

impl VerifySelf for DMXMode {
    fn verify_self(&mut self, roots: &mut VerificationRoots) -> Result<Vec<String>, String> {
        let mut warnings = Vec::new();
        let mut dmx_offsets = HashSet::new();

        let Some(geometries) = roots.get_mut_from_type::<FixtureType>()
            .map(|fixture_type| &fixture_type.geometries.geometries)
        else {
            unreachable!();
        };

        let refs = self.geometry.get_referenced_geometries(geometries);
        warnings.extend(refs.warnings.clone());
        println!("Hier!");
        println!("{}", refs);

        let mut non_native_channels = Vec::new();

        for dmx_channel in &self.dmx_channels.dmx_channels {
            Self::verify_channel(
                &dmx_channel, &self.geometry, geometries, &mut dmx_offsets, &mut non_native_channels,
            )?;

        }

        Self::verify_referenced_channels(&non_native_channels, &*refs.geometries, &mut dmx_offsets)?;

        println!("{:?}", dmx_offsets);

        Ok(warnings)
    }
}

impl VerifySelf for LogicalChannel {
    fn verify_self(&mut self, _roots: &mut VerificationRoots) -> Result<Vec<String>, String> {
        let mut warnings = Vec::new();


        warnings.push("Oh no, isn't implemented".into());

        Ok(warnings)
    }
}
