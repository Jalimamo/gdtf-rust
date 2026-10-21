mod verify_data_types;
mod verify_structure;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::io::{Cursor};
use bytes::Bytes;
use serde::{Deserialize, Deserializer};
use zip::ZipArchive;
use crate::parsing_helpers::verification::InitAtVerification::*;
use crate::parsing_helpers::verification::VerifyWarning::*;

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
        -> Result<Vec<VerifyWarning>, String>
    {Ok(Vec::new())}

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait VerifySelf:Verify {
    fn verify_self(&mut self, _roots: &mut VerificationRoots) -> Result<Vec<VerifyWarning>, String> {Ok(Vec::new())}
}

#[derive(Clone)]
pub enum VerifyWarning {
    IgnoredExcessInformation(String),
    ValueCorrected(String),
    FallbackUsed(String),
    InvalidInformationDiscarded(String),
    ProcessingSkipped(String),
    ConstraintIgnored(String),
}

impl fmt::Display for VerifyWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IgnoredExcessInformation(reason) =>
                write!(f, "Ignored Excess Information: {}", reason),
            ValueCorrected(reason) =>
                write!(f, "Value Corrected: {}", reason),
            FallbackUsed(reason) =>
                write!(f, "Fallback Used: {}", reason),
            InvalidInformationDiscarded(reason) =>
                write!(f, "Invalid Information Discarded: {}", reason),
            ProcessingSkipped(reason) =>
                write!(f, "Processing Skipped: {}", reason),
            ConstraintIgnored(reason) =>
                write!(f, "Constraint Ignored: {}", reason),
        }
    }
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