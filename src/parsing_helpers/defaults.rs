use arrayvec::ArrayVec;
use crate::parsing_helpers::{ColorCie, DMXBreak, DMXOffset, DmxValue, Float, HighlightValue, Matrix};
use crate::parsing_helpers::DmxValue::Mirroring;
use crate::parsing_helpers::names_and_references::Name;

pub fn default_dmx_break() -> DMXBreak {
    DMXBreak::Value(1)
}

pub fn default_dmx_offset() -> DMXOffset { DMXOffset(ArrayVec::new()) }

pub fn default_highlight_value() -> HighlightValue { HighlightValue(None) }

pub fn default_name() -> Name { Name::new("".into()).unwrap() }

pub fn default_color_space_name() -> Name { Name::new("Default".into()).unwrap() }

pub fn default_color_temperature() -> Float { 6_000.0 }

pub fn default_color_rendering_index() -> u8 { 100 }

pub fn default_connector_length() -> Float { 0.0 }

pub fn default_low_operating_temperature() -> Float { 0.0 }

pub fn default_high_operating_temperature() -> Float { 40.0 }

pub fn default_weight() -> Float { 0.0 }

pub fn default_leg_height() -> Float { 0.0 }

pub fn default_position() -> Matrix { Matrix { rows: [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
],}}

pub fn default_power_consumption() -> Float { 1000.0 }

pub fn default_luminous_flux() -> Float { 10000.0 }

pub fn default_beam_angle() -> Float { 25.0 }

pub fn default_field_angle() -> Float { 25.0 }

pub fn default_throw_ratio() -> Float { 1.0 }

pub fn default_rectangle_ratio() -> Float { 1.7777 }

pub fn default_beam_radius() -> Float { 0.05 }

pub fn default_snap() -> bool { false }

pub fn default_mib_fade() -> Float { 0.0 }

pub fn default_dmx_change_time_limit() -> Float { 0.0 }

pub fn default_channel_function_dmx_from() -> DmxValue {Mirroring {value: 0, byte_count: 1}}

pub fn default_channel_function_physical_from() -> Float { 0.0 }

pub fn default_channel_function_physical_to() -> Float { 1.0 }

pub fn default_channel_function_real_fader() -> Float { 0.0 }

pub fn default_channel_function_real_acceleration() -> Float { 0.0 }

pub fn default_mode_master_from() -> DmxValue {Mirroring {value: 0, byte_count: 1}}

pub fn default_mode_master_to() -> DmxValue {Mirroring {value: 0, byte_count: 1}}

pub fn default_channel_set_from() -> DmxValue {Mirroring {value: 0, byte_count: 1}}
pub fn default_ft_macro_step_duration() -> Float { 1.0 }

pub fn default_color_cie() -> ColorCie {
    ColorCie {
        x: 0.3127,
        y: 0.3290,
        yy: 100.0,
    }
}
