use std::collections::HashSet;
use crate::gdtf_xml_scheme::dmx_modes::{DMXMode, LogicalChannel};
use crate::gdtf_xml_scheme::FixtureType;
use crate::gdtf_xml_scheme::geometries::{
    BeamGeometry, Geometries, GeometryItem, GeometryReference, InventoryGeometry, LaserGeometry, StructureGeometry,
    SupportGeometry, WiringObjectChild, WiringObjectGeometry
};
use crate::gdtf_xml_scheme::physical_descriptions::{Emitter, Filter, PhysicalDescriptions};
use crate::parsing_helpers::{
    BeamType, LaserColorType, StructureCrossSectionType, SupportType, WiringObjectComponentType,
};
use crate::parsing_helpers::defaults::{
    default_beam_angle, default_field_angle, default_rectangle_ratio, default_throw_ratio
};
use crate::parsing_helpers::names_and_references::{GeometryName, Name, TopLevelGeometryName, Named};
use crate::parsing_helpers::verification::{VerificationRoots, VerifySelf, VerifyWarning};
use crate::parsing_helpers::verification::InitAtVerification::Verified;
use crate::parsing_helpers::verification::VerifyWarning::{
    ConstraintIgnored, IgnoredExcessInformation, InvalidInformationDiscarded
};

impl VerifySelf for Emitter {
    fn verify_self(&mut self, _roots: &mut VerificationRoots) -> Result<Vec<VerifyWarning>, String> {
        let mut warnings: Vec<VerifyWarning> = Vec::new();

        if self.color.is_none() && self.dominant_wave_length.is_none() {
            return Err(format!(
                "Emitter '{}' is missing required 'DominantWaveLength' because 'Color' is omitted.",
                self.name
            ));
        }

        self.measurements.retain(|measurement| {
            if measurement.luminous_intensity.is_none() {
                warnings.push(InvalidInformationDiscarded(format!(
                    "Emitter {} has a measurement without luminous intensity", self.name
                )));

                return false;
            }

            if measurement.physical_unit.0.eq(&0.0) {
                warnings.push(InvalidInformationDiscarded(format!(
                    "Emitter '{}' defines a measurement at physical percentage 0. This is redundant and contradicts \
                     DIN SPEC 15800, but processing will continue.", self.name
                )));
                return false;
            }

            if measurement.transmission.is_some() {
                warnings.push(IgnoredExcessInformation(format!(
                    "Emitter {} has a measurement with transmission, which will be ignored", self.name
                )))
            }

            true
        });

        Ok(warnings)
    }
}

impl VerifySelf for Filter {
    fn verify_self(&mut self, _roots: &mut VerificationRoots) -> Result<Vec<VerifyWarning>, String> {
        let mut warnings = Vec::new();

        self.measurements.retain(|measurement| {
            if measurement.transmission.is_none() {
                warnings.push(InvalidInformationDiscarded(format!(
                    "Filter {} has a measurement without transmission", self.name
                )));
                return false;
            }

            if measurement.luminous_intensity.is_some() {
                warnings.push(IgnoredExcessInformation(format!(
                    "Filter {} has a measurement with luminous intensity, which will be ignored", self.name
                )));
            }

            true
        });

        Ok(warnings)
    }
}

impl VerifySelf for PhysicalDescriptions {
    fn verify_self(&mut self, _roots: &mut VerificationRoots) -> Result<Vec<VerifyWarning>, String> {
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
    fn verify_self(&mut self, _roots: &mut VerificationRoots) -> Result<Vec<VerifyWarning>, String> {
        let mut warnings = Vec::new();

        match self.beam_type {
            BeamType::Wash | BeamType::Spot | BeamType::PC | BeamType::Fresnel | BeamType::Glow => {
                let beam_angle = *self.beam_angle.verify_or_default(default_beam_angle());
                let field_angle = *self.field_angle.verify_or_default(default_field_angle());

                if beam_angle > field_angle {
                    return Err(format!(
                        "Beam '{}' defines a BeamAngle ({}) that is greater than its FieldAngle ({}). This is \
                        physically invalid.", self.name, beam_angle, field_angle
                    ))
                }

                if self.throw_ratio.discard_unverified() {
                    warnings.push(IgnoredExcessInformation(format!(
                        "Beam '{}' defines a ThrowRatio, which is only valid for BeamType 'Rectangle'. The value will \
                         be ignored and removed.", self.name
                    )));
                }

                if self.rectangle_ratio.discard_unverified() {
                    warnings.push(IgnoredExcessInformation(format!(
                        "Beam '{}' defines a RectangleRatio, which is only valid for BeamType 'Rectangle'. The value \
                         will be ignored and removed.", self.name
                    )));
                }
            }

            BeamType::Rectangle => {
                self.throw_ratio.verify_or_default(default_throw_ratio());
                self.rectangle_ratio.verify_or_default(default_rectangle_ratio());

                if self.beam_angle.discard_unverified() {
                    warnings.push(IgnoredExcessInformation(format!(
                        "Beam '{}' of type 'Rectangle' defines a BeamAngle. This property is ignored for rectangular \
                         beams.", self.name
                    )));
                }

                if self.field_angle.discard_unverified() {
                    warnings.push(IgnoredExcessInformation(format!(
                        "Beam '{}' of type 'Rectangle' defines a FieldAngle. This property is ignored for rectangular \
                         beams.", self.name
                    )));
                }

            }
            BeamType::None => {
                if self.beam_angle.discard_unverified() {
                    warnings.push(IgnoredExcessInformation(format!(
                        "Beam '{}' of type 'None' defines a BeamAngle, which will be ignored.", self.name
                    )));
                }
                if self.field_angle.discard_unverified() {
                    warnings.push(IgnoredExcessInformation(format!(
                        "Beam '{}' of type 'None' defines a FieldAngle, which will be ignored.", self.name
                    )));
                }
                if self.throw_ratio.discard_unverified() {
                    warnings.push(IgnoredExcessInformation(format!(
                        "Beam '{}' of type 'None' defines a ThrowRatio, which will be ignored.", self.name
                    )));
                }
                if self.rectangle_ratio.discard_unverified() {
                    warnings.push(IgnoredExcessInformation(format!(
                        "Beam '{}' of type 'None' defines a RectangleRatio, which will be ignored.", self.name
                    )));
                }
            }
        }

        Ok(warnings)
    }
}

impl VerifySelf for Geometries {
    // fn verify_self(&mut self, _roots: &mut VerificationRoots) -> Result<Vec<VerifyWarning>, String> {
    //     let mut seen = HashSet::new();
    //     collect_geometry_names(self.geometries.iter(), &mut seen)?;
    //     Ok(Vec::new())
    // }
    fn verify_self(&mut self, _roots: &mut VerificationRoots) -> Result<Vec<VerifyWarning>, String> {
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
    fn verify_self(&mut self, _roots: &mut VerificationRoots) -> Result<Vec<VerifyWarning>, String> {
        let mut warnings = Vec::new();

        match self.color_type {
            LaserColorType::RGB => {
                if self.color.discard_unverified() {
                    warnings.push(IgnoredExcessInformation(format!(
                        "Laser '{}' defines a 'Color' value, but its ColorType is 'RGB'. The 'Color' attribute is \
                        only required for 'SingleWaveLength' and will be ignored.", self.name
                    )));
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
            return Err(format!(
                "Laser '{}' defines a BeamDivergenceMin ({}) that is greater than its BeamDivergenceMax ({}). \
                 This is physically invalid.",self.name, min, max
            ))
        }

        Ok(warnings)
    }
}

impl VerifySelf for WiringObjectGeometry {
    fn verify_self(&mut self, roots: &mut VerificationRoots) -> Result<Vec<VerifyWarning>, String> {
        let mut warnings = Vec::new();

        if self.component_type != WiringObjectComponentType::Consumer {
            if self.electrical_payload.discard_unverified() {
                warnings.push(IgnoredExcessInformation(format!(
                    "WiringObject '{}' defines 'ElectricalPayload', but ComponentType is not 'Consumer'. \
                    Property will be ignored and removed.", self.name
                )));
            }
            if self.voltage_range_max.discard_unverified() | self.voltage_range_min.discard_unverified() {
                warnings.push(IgnoredExcessInformation(format!(
                    "WiringObject '{}' defines voltage range properties, but ComponentType is not 'Consumer'. \
                    Properties will be ignored and removed.", self.name
                )));
            }
            if self.frequency_range_max.discard_unverified() | self.frequency_range_min.discard_unverified() {
                warnings.push(IgnoredExcessInformation(format!(
                    "WiringObject '{}' defines frequency range properties, but ComponentType is not 'Consumer'. \
                    Properties will be ignored and removed.", self.name
                )));
            }
            if self.cos_phi.discard_unverified() {
                warnings.push(IgnoredExcessInformation(format!(
                    "WiringObject '{}' defines 'CosPhi', but ComponentType is not 'Consumer'. \
                    Property will be ignored and removed.", self.name
                )));
            }
        } else {
            self.electrical_payload.make_verified();
            self.voltage_range_min.make_verified();
            self.voltage_range_max.make_verified();
            self.frequency_range_min.make_verified();
            self.frequency_range_max.make_verified();
            if let (Verified(min), Verified(max)) = (self.voltage_range_min, self.voltage_range_max) {
                if min > max {
                    return Err(format!(
                        "WiringObject '{}' defines VoltageRangeMin ({}) greater than VoltageRangeMax ({}). This is \
                         physically invalid.", self.name, min, max
                    ));
                }
            }
            if let (Verified(min), Verified(max)) = (self.frequency_range_min, self.frequency_range_max) {
                if min > max {
                    return Err(format!(
                        "WiringObject '{}' defines FrequencyRangeMin ({}) greater than FrequencyRangeMax ({}). \
                         This is physically invalid.", self.name, min, max
                    ));
                }
            }
        }

        if self.component_type != WiringObjectComponentType::PowerSource {
            if self.max_payload.discard_unverified() {
                warnings.push(IgnoredExcessInformation(format!(
                    "WiringObject '{}' defines 'MaxPayload', but ComponentType is not 'PowerSource'. Property will be \
                     ignored and removed.", self.name
                )));
            }
            if self.voltage.discard_unverified() {
                warnings.push(IgnoredExcessInformation(format!(
                    "WiringObject '{}' defines 'Voltage', but ComponentType is not 'PowerSource'. Property will be \
                    ignored and removed.", self.name
                )));
            }
        } else {
            self.max_payload.make_verified();
            self.voltage.make_verified();
        }

        if self.component_type != WiringObjectComponentType::Fuse {
            if self.fuse_current.discard_unverified() {
                warnings.push(IgnoredExcessInformation(format!(
                    "WiringObject '{}' defines 'FuseCurrent', but ComponentType is not 'Fuse'. Property will be \
                    ignored and removed.", self.name
                )));
            }
            if self.fuse_rating.discard_unverified() {
                warnings.push(IgnoredExcessInformation(format!(
                    "WiringObject '{}' defines 'FuseRating', but ComponentType is not 'Fuse'. Property will be \
                    ignored and removed.", self.name
                )));
            }
        } else {
            self.fuse_current.make_verified();
            self.fuse_rating.make_verified();
        }

        let pin_count_this = self.pin_count;
        for child in &self.children {
            if let WiringObjectChild::PinPatch(patch) = child {
                if patch.from_pin > pin_count_this || patch.from_pin < 1 {
                    return Err(format!(
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
                        return Err(format!(
                            "WiringObject '{}' contains a PinPatch referencing ToPin '{}', which exceeds the \
                            allowed PinCount range (1-{}) of target '{}'.",
                            self.name, patch.to_pin, pin_count_other, patch.to_wiring_object
                        ));
                    } else {
                        return Err(format!(
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
    fn verify_self(&mut self, _roots: &mut VerificationRoots) -> Result<Vec<VerifyWarning>, String> {
        if self.count < 0 {
            Err(format!("WiringObject '{}' contains a count of less than 0. This isn't technically against the \
            Norm, but it doesnt make any sense. How do you have {} of something?", self.name, self.count))
        } else {
            Ok(Vec::new())
        }
    }
}

impl VerifySelf for StructureGeometry {
    fn verify_self(&mut self, _roots: &mut VerificationRoots) -> Result<Vec<VerifyWarning>, String> {
        let mut warnings = Vec::new();

        match self.cross_section_type {
            StructureCrossSectionType::Tube => {
                if self.truss_cross_section.discard_unverified() {
                    warnings.push(IgnoredExcessInformation(format!(
                        "Structure '{}' defines a 'TrussCrossSection', but its CrossSectionType is 'Tube'. The \
                        property is only valid for trusses and will be removed.", self.name
                    )));
                }
                self.cross_section_height.make_verified();
                self.cross_section_wall_thickness.make_verified();
            }
            StructureCrossSectionType::TrussFramework => {
                if self.cross_section_height.discard_unverified() {
                    warnings.push(IgnoredExcessInformation(format!(
                        "Structure '{}' defines a 'CrossSectionHeight', but its CrossSectionType is 'TrussFramework'. \
                         The property is only valid for tubes and will be removed.", self.name
                    )));
                }

                if self.cross_section_wall_thickness.discard_unverified() {
                    warnings.push(IgnoredExcessInformation(format!(
                        "Structure '{}' defines a 'CrossSectionWallThickness', but its CrossSectionType is \
                        'TrussFramework'. The property is only valid for tubes and will be removed.", self.name
                    )));
                }

                self.truss_cross_section.make_verified();
            }
        }

        Ok(warnings)
    }
}

impl VerifySelf for SupportGeometry {
    fn verify_self(&mut self, _roots: &mut VerificationRoots) -> Result<Vec<VerifyWarning>, String> {
        let mut warnings = Vec::new();

        match self.support_type {
            SupportType::GroundSupport => {
                // Rope-Eigenschaften bei GroundSupports bereinigen
                if self.rope_cross_section.discard_unverified() {
                    warnings.push(IgnoredExcessInformation(format!(
                        "Support '{}' defines a 'RopeCrossSection', but its SupportType is 'GroundSupport'. This \
                        property is only valid for ropes and will be removed.", self.name
                    )));
                }

                if self.rope_offset.discard_unverified() {
                    warnings.push(IgnoredExcessInformation(format!(
                        "Support '{}' defines a 'RopeOffset', but its SupportType is 'GroundSupport'. This property is \
                         only valid for ropes and will be removed.", self.name
                    )));
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
                    warnings.push(IgnoredExcessInformation(format!(
                        "Support '{}' defines one or more Resistance properties, but its SupportType is 'Rope'. These \
                        properties are only valid for GroundSupports and will be removed.", self.name
                    )));
                }

                self.rope_cross_section.make_verified();
                self.rope_offset.make_verified();
            }
        }

        Ok(warnings)
    }
}

impl VerifySelf for GeometryReference {
    fn verify_self(&mut self, _roots: &mut VerificationRoots) -> Result<Vec<VerifyWarning>, String> {
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
    fn verify_self(&mut self, roots: &mut VerificationRoots) -> Result<Vec<VerifyWarning>, String> {
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
    fn verify_self(&mut self, _roots: &mut VerificationRoots) -> Result<Vec<VerifyWarning>, String> {
        let mut warnings = Vec::new();


        warnings.push(ConstraintIgnored("Oh no, isn't implemented".into()));

        Ok(warnings)
    }
}
