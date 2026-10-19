use std::collections::HashSet;
use crate::gdtf_xml_scheme::dmx_modes::{DMXChannel, DMXMode};
use crate::gdtf_xml_scheme::geometries::GeometryItem;
use crate::parsing_helpers::{DMXAddress, DMXBreak};
use crate::parsing_helpers::names_and_references::{Named, ReferencedGeometry, TopLevelGeometryName};

impl DMXMode {
    pub(crate) fn verify_channel<'a>(
        channel: &'a DMXChannel, mode_geometry: &TopLevelGeometryName, geometries: &Vec<GeometryItem>,
        dmx_offsets: &mut HashSet<(u32, DMXAddress)>, non_native_channels: &mut Vec<&'a DMXChannel>
    ) -> Result<(), String> {

        let mut logical_channels = HashSet::new();

        if channel.geometry.is_descendant_of(mode_geometry, geometries)
            && let DMXBreak::Value(dmx_break) = channel.dmx_break
        {
            for offset in &channel.offset {
                if !dmx_offsets.insert((dmx_break, DMXAddress::from_u32(*offset))) {
                    return Err(format!("DMX offset {} occurs more than once", offset))
                }
            }
        } else {
            non_native_channels.push(channel);
        }


        for logical_channel in &channel.logical_channels {
            if !logical_channels.insert((channel.geometry.as_str(), logical_channel.name())) {
                return Err(format!(
                    "Logical channel attribute '{}' is not unique for geometry '{}' within this DMX mode ",
                    logical_channel.name(), channel.name()
                ));
            }
        }

        if channel.logical_channels.is_empty() {
            return Err(format!(
                "DMX-Channel {} has no LogicalChannel, but needs at least 1", channel.name()
            ))
        }

        Ok(())
    }

    pub(crate) fn verify_referenced_channels(
        channels: &[&DMXChannel], geometries: &[ReferencedGeometry],
        dmx_offsets: &mut HashSet<(u32, DMXAddress)>
    ) -> Result<(), String> {
        for geometry in geometries {
            for channel in channels.iter()
                .filter(|channel|
                    channel.geometry.eq(&geometry.geometry)
                )
            {
                for reference in geometry.dmx_break.iter()
                    .filter(|dmx_break|
                        dmx_break.dmx_break.as_dmx_break() == channel.dmx_break
                    )
                {

                    let dmx_break_value = reference.dmx_break.get_value();

                    for channel_offset in &channel.offset {
                        let offset = reference.dmx_offset + *channel_offset;
                        if !dmx_offsets.insert((dmx_break_value, offset)) {
                            return Err(format!("DMX offset {:?} occurs more than once", offset))
                        }
                    }
                }

            }
        }
        Ok(())
    }
}