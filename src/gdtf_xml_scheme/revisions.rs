use serde::Deserialize;
use zip::ZipArchive;
use std::io::Cursor;
use gdtf_derive::GdtfVerify;
use crate::parsing_helpers::{GdtfDate, Uint, zero_as_none, empty_string_as_none};
use crate::parsing_helpers::verification::Verify;


#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct Revisions {
    #[serde(rename = "Revision", default)]
    pub revisions: Vec<Revision>
}

#[derive(Debug, Clone, Deserialize, GdtfVerify)]
#[serde(deny_unknown_fields)]
pub struct Revision {
    #[serde(rename = "@Text")]
    pub text: String,
    #[serde(rename = "@Date")]
    pub date: GdtfDate,
    #[serde(rename = "@UserID", default, deserialize_with = "zero_as_none")]
    pub user_id: Option<Uint>,
    #[serde(rename = "@ModifiedBy", default, deserialize_with = "empty_string_as_none")]
    pub modified_by: Option<String>,
}