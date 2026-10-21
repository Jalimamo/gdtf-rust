use std::any::Any;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io::Cursor;
use std::marker::PhantomData;
use bytes::Bytes;
use serde::{Deserialize, Deserializer};
use serde::de::IntoDeserializer;
use zip::ZipArchive;
use crate::gdtf_xml_scheme::attribute_definitions::{ActivationGroup, Attribute, Feature, FeatureGroup, SubPhysicalUnit};
use crate::gdtf_xml_scheme::dmx_modes::{ChannelFunction, DMXChannel, DMXMode, LogicalChannel};
use crate::gdtf_xml_scheme::FixtureType;
use crate::gdtf_xml_scheme::geometries::{GeometryItem, GeometryReference, GeometryReferenceDmxBreak, LaserChildren, WiringObjectChild};
use crate::gdtf_xml_scheme::models::{Model, Models};
use crate::gdtf_xml_scheme::physical_descriptions::{ColorSpace, DmxProfile, Emitter, Filter, Gamut};
use crate::gdtf_xml_scheme::wheels::Wheel;
use crate::parsing_helpers::{is_invalid_but_gdtf_name_we_allow_it, is_valid_gdtf_name_char};
use crate::parsing_helpers::names_and_references::PathResult::*;
use crate::parsing_helpers::verification::{Verify, VerificationRoots, VerifyWarning};
use crate::parsing_helpers::verification::VerifyWarning::{ConstraintIgnored, FallbackUsed, ProcessingSkipped};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name(String);

impl <'de> Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        let trimmed = s.trim();

        Self::new(trimmed).map_err(|_| serde::de::Error::custom(format!(
            "Invalid GDTF name: Contains restricted characters according to DIN SPEC 15800, Annex C. Got {}",
            trimmed
        )))
    }
}

impl Verify for Name {
    fn verify(&mut self, _roots: &mut VerificationRoots, _archive: &mut ZipArchive<Cursor<Bytes>>)
              -> Result<Vec<VerifyWarning>, String>
    {
        let mut warnings = vec![];
        for char in self.0.chars().into_iter() {
            if is_invalid_but_gdtf_name_we_allow_it(char) {
                warnings.push(ConstraintIgnored(format!("Invalid Char in GDTF name: '{}'", char)));
            }
        }
        Ok(warnings)
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {self}
}

impl Display for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Name {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn new(name: &str) -> Result<Self,()> {
        if !name.chars().all(is_valid_gdtf_name_char) {
            Err(())
        } else {
            Ok(Name(name.to_string()))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelName(pub Name);

impl<'de> Deserialize<'de> for ModelName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let name = Name::deserialize(deserializer)?;
        Ok(ModelName(name))
    }
}

impl Display for ModelName {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ModelName {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn get_model<'a>(&self, models: &'a Models) -> Option<&'a Model> {
        models.models.iter().find(|m| m.name.as_str() == self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeometryName(pub Name);

impl<'de> Deserialize<'de> for GeometryName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let name = Name::deserialize(deserializer)?;
        Ok(GeometryName(name))
    }
}

impl Display for GeometryName {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl GeometryName {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn from_geometry_item(item: &GeometryItem) -> Self {
        GeometryName(Name(item.name()))
    }

    pub fn get_this_geometry<'a>(&self, geometries: &'a [GeometryItem]) -> Option<&'a GeometryItem> {
        Self::get_geometry(Box::new(geometries.iter()), self.as_str())
    }

    fn get_geometry<'a>(mut geometries: impl Iterator<Item=&'a GeometryItem>, name: &str) -> Option<&'a GeometryItem> {
        geometries.find_map(|geometry| {
            if geometry.name() == name {
                Some(geometry)
            } else {
                Self::get_geometry(Self::children(geometry), name)
            }
        })
    }



    pub(super) fn children<'a>(geometry: &'a GeometryItem) -> Box<dyn Iterator<Item = &'a GeometryItem> + 'a> {
        match geometry {
            GeometryItem::Geometry(x) => Box::new(x.children.iter()),
            GeometryItem::Axis(x) => Box::new(x.children.iter()),
            GeometryItem::FilterBeam(x) => Box::new(x.children.iter()),
            GeometryItem::FilterColor(x) => Box::new(x.children.iter()),
            GeometryItem::FilterGobo(x) => Box::new(x.children.iter()),
            GeometryItem::FilterShaper(x) => Box::new(x.children.iter()),
            GeometryItem::Beam(x) => Box::new(x.children.iter()),
            GeometryItem::MediaServerLayer(x) => Box::new(x.children.iter()),
            GeometryItem::MediaServerCamera(x) => Box::new(x.children.iter()),
            GeometryItem::MediaServerMaster(x) => Box::new(x.children.iter()),
            GeometryItem::Display(x) => Box::new(x.children.iter()),
            GeometryItem::GeometryReference(_) => Box::new(std::iter::empty()),
            GeometryItem::Laser(x) => Box::new(x.children.iter().filter_map(|c| match c {
                LaserChildren::Protocol(_) => None,
                LaserChildren::Geometry(y) => Some(y),
            })),
            GeometryItem::WiringObject(x) => Box::new(x.children.iter().filter_map(|c| match c {
                WiringObjectChild::PinPatch(_) => None,
                WiringObjectChild::Geometry(y) => Some(y),
            })),
            GeometryItem::Inventory(x) => Box::new(x.children.iter()),
            GeometryItem::Structure(x) => Box::new(x.children.iter()),
            GeometryItem::Support(x) => Box::new(x.children.iter()),
            GeometryItem::Magnet(x) => Box::new(x.children.iter()),
        }
    }

    pub fn is_descendant_of(&self, parent: &TopLevelGeometryName, geometries: &[GeometryItem]) -> bool {
        let Some(parent_geometry) = parent.get_this_geometry(geometries) else {
            return false;
        };

        Self::contains_geometry(
            Self::children(parent_geometry),
            self.as_str(),
        )
    }

    fn contains_geometry<'a>(mut geometries: impl Iterator<Item=&'a GeometryItem>, name: &str) -> bool {
        geometries.any(|g| {
            g.name() == name ||
                Self::contains_geometry(
                    Self::children(g),name
                )
        })
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct TopLevelGeometryName(pub(crate) Name);

impl<'de> Deserialize<'de> for TopLevelGeometryName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let name = Name::deserialize(deserializer)?;
        Ok(TopLevelGeometryName(name))
    }
}

impl Display for TopLevelGeometryName {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TopLevelGeometryName {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn get_this_geometry<'a>(&self, geometries: &'a [GeometryItem]) -> Option<&'a GeometryItem> {
        geometries.iter().find(|geo|
            geo.name() == self.as_str()
        )
    }

    pub fn get_referenced_geometries(&self, geometries: &[GeometryItem]) -> ReferencedGeometries {
        let mut result = ReferencedGeometries::default();

        let Some(top_level) = self.get_this_geometry(geometries) else {
            unreachable!();
        };

        let mut visited = vec![self.clone()];

        Self::visit_geometry(top_level, geometries, None, &mut visited, &mut result);


        result
    }

    fn visit_geometry(
        geometry: &GeometryItem, all_geometries: &[GeometryItem], current_breaks: Option<&[GeometryReferenceDmxBreak]>,
        visited: &mut Vec<TopLevelGeometryName>, result: &mut ReferencedGeometries
    ) {
        match geometry {
            GeometryItem::GeometryReference(reference) => {
                println!("{:?}", reference);
                Self::visit_reference(
                    reference, all_geometries, current_breaks, visited, result
                );
            }

            _ => {
                if let Some(current_breaks) = current_breaks {
                    result.geometries.push(ReferencedGeometry{
                        geometry: GeometryName::from_geometry_item(geometry),
                        dmx_break: current_breaks.to_vec(),
                    });
                }

                for child in GeometryName::children(geometry) {
                    Self::visit_geometry(child, all_geometries, current_breaks, visited, result);
                }
            }
        }
    }

    fn visit_reference(
        reference: &GeometryReference, all_geometries: &[GeometryItem],
        current_breaks: Option<&[GeometryReferenceDmxBreak]>, visited: &mut Vec<TopLevelGeometryName>,
        result: &mut ReferencedGeometries
    ) {

        if visited.contains(&reference.geometry) {
            result.warnings.push(ProcessingSkipped(format!(
                "Circular GeometryReference detected: '{}'.",
                reference.geometry,
            )));

            return;
        }

        let Some(referenced_geometry) = reference.geometry.get_this_geometry(all_geometries) else {
            unreachable!();
        };

        let combined_breaks = combine_breaks(
            current_breaks, &reference.dmx_break, &mut result.warnings
        );

        visited.push(reference.geometry.clone());

        Self::visit_geometry(referenced_geometry, all_geometries, Some(&combined_breaks), visited, result);

        visited.pop();
    }
}

#[derive(Debug, Clone)]
pub struct ReferencedGeometry {
    pub geometry: GeometryName,
    pub dmx_break: Vec<GeometryReferenceDmxBreak>,
}

pub struct ReferencedGeometries {
    pub geometries: Vec<ReferencedGeometry>,
    pub warnings: Vec<VerifyWarning>,
}

impl Default for ReferencedGeometries {
    fn default() -> Self {
        Self {
            geometries: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

impl Display for ReferencedGeometries {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for geometry in &self.geometries {
            writeln!(f, "Geometry: {}", geometry.geometry)?;

            for dmx_break in &geometry.dmx_break {
                writeln!(f,
                         "  DMX break {:?}: Offset {:?}",
                         dmx_break.dmx_break,
                         dmx_break.dmx_offset
                )?;
            }
        }

        if !self.warnings.is_empty() {
            writeln!(f, "Warnings:")?;

            for warning in &self.warnings {
                writeln!(f, "   - {}", warning)?;
            }
        }

        Ok(())
    }
}

fn combine_breaks(
    old: Option<&[GeometryReferenceDmxBreak]>, new: &[GeometryReferenceDmxBreak], warnings: &mut Vec<VerifyWarning>
) -> Vec<GeometryReferenceDmxBreak> {

    let Some(old) = old else {
        return new.to_vec();
    };

    let mut combined = Vec::new();

    for new_break in new {
        let Some(old_break) = old.iter().find(|old_break| {
            old_break.dmx_break == new_break.dmx_break
        }) else {
            warnings.push(FallbackUsed(format!(
                "DMX Break {} with Offset {:?} was ignored, because there was no matching Break in previous Reference",
                new_break.dmx_break, new_break.dmx_offset
            )));

            continue;
        };

        combined.push(GeometryReferenceDmxBreak {
            dmx_offset: old_break.dmx_offset + new_break.dmx_offset,
            dmx_break: new_break.dmx_break.clone()
        });
    }

    combined
}

pub(crate) trait Named {
    fn name(&self) -> String;
}

impl Named for ActivationGroup {
    fn name(&self) -> String {self.name.to_string()}
}

impl Named for FeatureGroup {
    fn name(&self) -> String {self.name.to_string()}
}

impl Named for Feature {
    fn name(&self) -> String {self.name.to_string()}
}

impl Named for Attribute {
    fn name(&self) -> String {self.name.to_string()}
}

impl Named for Wheel {
    fn name(&self) -> String {self.name.to_string()}
}

impl Named for Emitter {
    fn name(&self) -> String {self.name.to_string()}
}

impl Named for Filter {
    fn name(&self) -> String {self.name.to_string()}
}

impl Named for Gamut {
    fn name(&self) -> String {self.name.to_string()}
}

impl Named for DmxProfile {
    fn name(&self) -> String {self.name.to_string()}
}

impl Named for Model {
    fn name(&self) -> String {self.name.to_string()}
}

impl Named for GeometryItem {
    fn name(&self) -> String {
        match self {
            GeometryItem::Geometry(x) => x.name.to_string(),
            GeometryItem::Axis(x) => x.name.to_string(),
            GeometryItem::FilterBeam(x) => x.name.to_string(),
            GeometryItem::FilterColor(x) => x.name.to_string(),
            GeometryItem::FilterGobo(x) => x.name.to_string(),
            GeometryItem::FilterShaper(x) => x.name.to_string(),
            GeometryItem::Beam(x) => x.name.to_string(),
            GeometryItem::MediaServerLayer(x) => x.name.to_string(),
            GeometryItem::MediaServerCamera(x) => x.name.to_string(),
            GeometryItem::MediaServerMaster(x) => x.name.to_string(),
            GeometryItem::Display(x) => x.name.to_string(),
            GeometryItem::GeometryReference(x) => x.name.to_string(),
            GeometryItem::Laser(x) => x.name.to_string(),
            GeometryItem::WiringObject(x) => x.name.to_string(),
            GeometryItem::Inventory(x) => x.name.to_string(),
            GeometryItem::Structure(x) => x.name.to_string(),
            GeometryItem::Support(x) => x.name.to_string(),
            GeometryItem::Magnet(x) => x.name.to_string(),
        }
    }
}

impl Named for DMXMode {
    fn name(&self) -> String {self.name.to_string()}
}

impl Named for DMXChannel {
    fn name(&self) -> String {
        let logical_channel_name = self.logical_channels.get(0)
            .as_ref().map(|logical_channel| logical_channel.attribute.to_string())
            .unwrap_or("".to_string());
        format!(
            "{}_{}", self.geometry.as_str(), logical_channel_name
        )
    }
}

impl Named for LogicalChannel {
    fn name(&self) -> String {
        self.attribute.to_string()
    }
}

impl Named for ChannelFunction {
    fn name(&self) -> String {self.name.to_string()}
}

impl Named for ColorSpace {
    fn name(&self) -> String {self.name.to_string()}
}

impl Named for SubPhysicalUnit {
    fn name(&self) -> String {self.unit_type.to_string()}
}

impl Named for GeometryReferenceDmxBreak {
    fn name(&self) -> String {self.dmx_break.to_string()}
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodePath<T: Verify>(pub Vec<String>, PhantomData<T>);

impl<'de, T: Verify> Deserialize<'de> for NodePath<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        let path = s.split('.').map(|p| p.to_string()).collect();
        Ok(NodePath(path, PhantomData))
    }
}

impl<T: Verify> NodePath<T> where FixtureType: ResolveNodePath<T> {
    pub fn get<'a>(
        &self, fixture_type: &'a FixtureType, root_data: <FixtureType as ResolveNodePath<T>>::RootData
    ) -> PathResult<&'a T> {
        let path = root_data.prepend_to_path(&self.0);

        fixture_type.resolve_path(&path)
    }

    pub fn get_from_roots<'a>(&self, roots: &'a mut VerificationRoots) -> PathResult<&'a T> {
        let root_data = match <<FixtureType as ResolveNodePath<T>>::RootData>::from_roots(roots) {
            Exists(data) => data,
            PathWrong => return PathWrong,
            Missing => return Missing,
            InvalidRoot => return InvalidRoot,
        };

        let Some(fixture_type) = roots.get_mut_from_type::<FixtureType>() else {

            println!("DILDO");

            return InvalidRoot;
        };

        self.get(fixture_type, root_data)
    }
}

impl<T: Verify> Display for NodePath<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.0.join("."))
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(transparent)]
pub struct OptionalNodePath<T: Verify>(pub Option<NodePath<T>>);

impl<T: Verify> Default for OptionalNodePath<T> {
    fn default() -> Self {
        OptionalNodePath(None)
    }
}

impl<T: Verify> Display for OptionalNodePath<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(ref node) => write!(f, "{}", node),
            None => write!(f, ""),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum ModeMasterNodePath {
    ToDMXChannel(NodePath<DMXChannel>),
    ToChannelFunction(NodePath<ChannelFunction>),
    #[default]
    Inactive
}

pub enum ModeMasterObject<'a> {
    DMXChannel(&'a DMXChannel),
    ChannelFunction(&'a ChannelFunction),
    Inactive
}

impl<'de> Deserialize<'de> for ModeMasterNodePath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        match s.split('.').count() {
            1 => {
                let path = NodePath::<DMXChannel>::deserialize(s.into_deserializer())?;
                Ok(ModeMasterNodePath::ToDMXChannel(path))
            },
            3 => {
                let path = NodePath::<ChannelFunction>::deserialize(s.into_deserializer())?;
                Ok(ModeMasterNodePath::ToChannelFunction(path))
            },
            _ => Ok(ModeMasterNodePath::Inactive)
        }
    }
}

impl ModeMasterNodePath {
    pub fn get<'a>(&self, roots: &'a mut VerificationRoots) -> PathResult<ModeMasterObject<'a>> {
        match self {
            Self::ToDMXChannel(path) =>
                path.get_from_roots(roots).map(ModeMasterObject::DMXChannel),
            Self::ToChannelFunction(path) =>
                path.get_from_roots(roots).map(ModeMasterObject::ChannelFunction),
            Self::Inactive =>
                Exists(ModeMasterObject::Inactive),
        }
    }
    
    pub fn path_string(&self) -> String {
        match self {
            Self::ToDMXChannel(path) => path.to_string(),
            Self::ToChannelFunction(path) => path.to_string(),
            Self::Inactive => "".to_string()
        }
    }
}




pub trait ResolveNodePath<T: Verify> {
    type RootData:RelativeNodeStartData;
    fn resolve_path(&self, path: &[String]) -> PathResult<&T>;
}

pub trait RelativeNodeStartData: Sized {
    fn from_roots(roots: &mut VerificationRoots) -> PathResult<Self>;
    fn prepend_to_path(self, path: &[String]) -> Vec<String>;
}
pub struct NoNodeRootData;
impl RelativeNodeStartData for NoNodeRootData {
    fn from_roots(_: &mut VerificationRoots) -> PathResult<Self> {
        Exists(NoNodeRootData)
    }
    fn prepend_to_path(self, path: &[String]) -> Vec<String> {
        path.to_vec()
    }
}

pub struct DmxModeNodeRootData(String);
impl RelativeNodeStartData for DmxModeNodeRootData {
    fn from_roots(roots: &mut VerificationRoots) -> PathResult<Self> {
        let node_name = roots
            .get_mut_from_type::<DMXMode>()    // 0: Gdtf, 1: FixtureType, 2: DMXModes, 3: DMXMode
            .map(|dmx_mode| dmx_mode.name.to_string());

        if node_name.is_none() {
            println!("PENIS")
        }

        node_name.map_or(InvalidRoot, |x |Exists(DmxModeNodeRootData(x)))
    }
    fn prepend_to_path(self, path: &[String]) -> Vec<String> {
        let mut result = vec![self.0];
        result.extend_from_slice(path);
        result
    }
}

pub struct AttributeNodeRootData(String);
impl RelativeNodeStartData for AttributeNodeRootData {
    fn from_roots(roots: &mut VerificationRoots) -> PathResult<Self> {
        let node_name = roots
            .get_mut_from_type::<ChannelFunction>()
            .map(|channel_function| channel_function.attribute.to_string());

        if node_name.is_none() {
            println!("Vagina")
        }

        node_name.map_or(InvalidRoot, |x |Exists(AttributeNodeRootData(x)))
    }
    fn prepend_to_path(self, path: &[String]) -> Vec<String> {
        let mut result = vec![self.0];
        result.extend_from_slice(path);
        result
    }
}


impl ResolveNodePath<ActivationGroup> for FixtureType {
    type RootData = NoNodeRootData;
    fn resolve_path(&self, path: &[String]) -> PathResult<&ActivationGroup> {
        let [target_name] = path else {
            return PathWrong
        };

        let found = self.attribute_definitions.activation_groups.iter()
            .flat_map(|activation_groups| activation_groups.activation_group.iter())
            .find(|activation_group| activation_group.name() == *target_name);

        found.map_or(Missing, Exists)

    }
}

impl ResolveNodePath<Feature> for FixtureType {
    type RootData = NoNodeRootData;
    fn resolve_path(&self, path: &[String]) -> PathResult<&Feature> {
        let [target_group, target_feature] = path else {
            return PathWrong
        };

        let found = self.attribute_definitions.feature_groups.feature_group.iter()
            .find(|group| group.name.as_str() == target_group).iter()
            .flat_map(|group| group.features.iter())
            .find(|feature| feature.name() == *target_feature);

        found.map_or(Missing, Exists)
    }
}

impl ResolveNodePath<Attribute> for FixtureType {
    type RootData = NoNodeRootData;
    fn resolve_path(&self, path: &[String]) -> PathResult<&Attribute> {
        let [target_name] = path else {
            return PathWrong
        };

        let found = self.attribute_definitions.attributes.attributes.iter()
            .find(|attribute| attribute.name.as_str() == target_name);

        found.map_or(Missing, Exists)
    }
}

impl ResolveNodePath<Filter> for FixtureType {
    type RootData = NoNodeRootData;
    fn resolve_path(&self, path: &[String]) -> PathResult<&Filter> {
        let [target_name] = path else {
            return PathWrong
        };

        let found = self.physical_descriptions.iter()
            .flat_map(|descriptions| descriptions.filters.iter())
            .flat_map(|filters| filters.filters.iter())
            .find(|filter| filter.name() == *target_name);

        found.map_or(Missing, Exists)
    }
}

impl ResolveNodePath<Emitter> for FixtureType {
    type RootData = NoNodeRootData;
    fn resolve_path(&self, path: &[String]) -> PathResult<&Emitter> {
        let [target_name] = path else {
            return PathWrong
        };

        let found = self.physical_descriptions.iter()
            .flat_map(|descriptions| descriptions.emitters.iter())
            .flat_map(|emitter| emitter.emitters.iter())
            .find(|emitter| emitter.name() == *target_name);

        found.map_or(Missing, Exists)
    }
}

impl ResolveNodePath<ChannelFunction> for FixtureType {
    type RootData = DmxModeNodeRootData;
    fn resolve_path(&self, path: &[String]) -> PathResult<&ChannelFunction> {
        let [
        target_dmx_mode,
        target_channel,
        target_logical_channel,
        target_channel_function
        ] = path else {
            return PathWrong
        };

        let found = self.dmx_modes.modes.iter()
            .find(|mode| {mode.name() == *target_dmx_mode}).iter()
            .flat_map(|mode| mode.dmx_channels.dmx_channels.iter())
            .find(|channel| channel.name() == *target_channel).iter()
            .flat_map(|channel| channel.logical_channels.iter())
            .find(|logical_channel| logical_channel.name() == *target_logical_channel).iter()
            .flat_map(|logical_channel| logical_channel.channel_functions.iter())
            .find(|channel_function| channel_function.name() == *target_channel_function);


        found.map_or(Missing, Exists)
    }
}

impl ResolveNodePath<Wheel> for FixtureType {
    type RootData = NoNodeRootData;
    fn resolve_path(&self, path: &[String]) -> PathResult<&Wheel> {
        let [target_name] = path else {
            return PathWrong
        };

        let found = self.wheels.iter()
            .flat_map(|wheels| wheels.wheels.iter())
            .find(|wheel| {wheel.name() == *target_name});

        found.map_or(Missing, Exists)
    }
}

impl ResolveNodePath<ColorSpace> for FixtureType {
    type RootData = NoNodeRootData;
    fn resolve_path(&self, path: &[String]) -> PathResult<&ColorSpace> {
        let [target_name] = path else {
            return PathWrong
        };

        let Some(descriptions) = self.physical_descriptions.as_ref() else {
            return Missing
        };

        let found = descriptions.additional_color_spaces.as_ref().into_iter()
            .flat_map(|additional| additional.color_spaces.iter())
            .chain(descriptions.color_space.iter())
            .find(|color_space| color_space.name() == *target_name);

        found.map_or(Missing, Exists)
    }
}

impl ResolveNodePath<Gamut> for FixtureType {
    type RootData = NoNodeRootData;
    fn resolve_path(&self, path: &[String]) -> PathResult<&Gamut> {
        let [target_name] = path else {
            return PathWrong
        };

        let found = self.physical_descriptions.iter()
            .flat_map(|descriptions| descriptions.gamuts.iter())
            .flat_map(|gamuts| gamuts.gamuts.iter())
            .find(|gamut| gamut.name() == *target_name);

        found.map_or(Missing, Exists)
    }
}

impl ResolveNodePath<DMXChannel> for FixtureType {
    type RootData = DmxModeNodeRootData;
    fn resolve_path(&self, path: &[String]) -> PathResult<&DMXChannel> {
        let [
        target_dmx_mode,
        target_channel
        ] = path else {
            return PathWrong
        };

        let found = self.dmx_modes.modes.iter()
            .find(|mode| {mode.name() == *target_dmx_mode}).iter()
            .flat_map(|mode| mode.dmx_channels.dmx_channels.iter())
            .find(|channel| channel.name() == *target_channel);

        found.map_or(Missing, Exists)
    }
}

impl ResolveNodePath<DmxProfile> for FixtureType {
    type RootData = NoNodeRootData;
    fn resolve_path(&self, path: &[String]) -> PathResult<&DmxProfile> {
        let [target_name] = path else {
            return PathWrong
        };

        let found = self.physical_descriptions.iter()
            .flat_map(|descriptions| descriptions.dmx_profiles.iter())
            .flat_map(|dmx_profiles| dmx_profiles.profiles.iter())
            .find(|dmx_profile| dmx_profile.name() == *target_name);

        found.map_or(Missing, Exists)
    }
}

impl ResolveNodePath<SubPhysicalUnit> for FixtureType {
    type RootData = AttributeNodeRootData;
    fn resolve_path(&self, path: &[String]) -> PathResult<&SubPhysicalUnit> {
        let [attribute_name, target_name] = path else {
            return PathWrong
        };

        let found = self.attribute_definitions.attributes.attributes.iter()
            .find(|attribute| attribute.name() == *attribute_name).iter()
            .flat_map(|attribute| attribute.sub_physical_unit.iter())
            .find(|sub_physical_unit| sub_physical_unit.name() == *target_name);

        found.map_or(Missing, Exists)
    }
}

impl ResolveNodePath<DMXMode> for FixtureType {
    type RootData = NoNodeRootData;
    fn resolve_path(&self, path: &[String]) -> PathResult<&DMXMode> {
        let [target_name] = path else {
            return PathWrong
        };

        let found = self.dmx_modes.modes.iter()
            .find(|mode| mode.name() == *target_name);

        found.map_or(Missing, Exists)
    }
}


pub enum PathResult<T> {
    Exists(T),
    PathWrong,
    Missing,
    InvalidRoot
}

impl<T> PathResult<T> {
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> PathResult<U> {
        match self {
            Exists(u) => Exists(f(u)),
            PathWrong => PathWrong,
            Missing => Missing,
            InvalidRoot => InvalidRoot,
        }
    }
}
