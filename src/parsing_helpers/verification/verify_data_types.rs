use std::any::Any;
use std::io::Cursor;
use bytes::Bytes;
use zip::ZipArchive;
use crate::gdtf_xml_scheme::FixtureType;
use crate::parsing_helpers::{ColorCie, DMXOffset, Float, GdtfDate, Gender, Int, Uint};
use crate::parsing_helpers::names_and_references::{GeometryName, ModeMasterNodePath, ModelName, NodePath, OptionalNodePath, ResolveNodePath, TopLevelGeometryName};
use crate::parsing_helpers::names_and_references::PathResult::{Exists, InvalidRoot, Missing, PathWrong};
use crate::parsing_helpers::resources::{Resource, ResourceType};
use crate::parsing_helpers::resources::ResourceError::{NoFileFoundError, UnzipError};
use crate::parsing_helpers::verification::{InitAtVerification, VerificationRoots, Verify, VerifyWarning};
use crate::parsing_helpers::verification::VerifyWarning::{ConstraintIgnored, FallbackUsed, ValueCorrected};

fn verify_items<'a, I>(items: I, roots: &mut VerificationRoots, archive: &mut ZipArchive<Cursor<Bytes>>)
                       -> Result<Vec<VerifyWarning>, String> where I: IntoIterator<Item=&'a mut dyn Verify>
{
    let mut warnings = Vec::new();

    for item in items {
        warnings.extend(item.verify(roots, archive)?);
    }

    Ok(warnings)
}

impl<T: Verify + 'static> Verify for Vec<T> {
    fn verify(&mut self, roots: &mut VerificationRoots, archive: &mut ZipArchive<Cursor<Bytes>>)
              -> Result<Vec<VerifyWarning>, String>
    {
        verify_items(self.iter_mut().map(|i| i as &mut dyn Verify), roots, archive)
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {self}
}

impl<T: Verify + 'static, const N: usize> Verify for [T; N] {
    fn verify(&mut self, roots: &mut VerificationRoots, archive: &mut ZipArchive<Cursor<Bytes>>)
              -> Result<Vec<VerifyWarning>, String>
    {
        verify_items(self.iter_mut().map(|i| i as &mut dyn Verify), roots, archive)
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {self}
}

impl<T: Verify + 'static> Verify for Option<T> {
    fn verify(&mut self, roots: &mut VerificationRoots, archive: &mut ZipArchive<Cursor<Bytes>>)
              -> Result<Vec<VerifyWarning>, String>
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
              -> Result<Vec<VerifyWarning>, String>
    {
        let mut warnings = Vec::new();

        if self.x < 0.0 || self.x > 1.0 {
            return Err(format!("CIExyY Error: x out of Range [0,1]. Got {}.", self.x))
        } else if self.y < 0.0 || self.y > 1.0 {
            return Err(format!("CIExyY Error: y out of Range [0,1]. Got {}.", self.y))
        } else if self.yy < 0.0 || self.yy > 100.0 {
            warnings.push(ValueCorrected(format!(
                "CIExyY Warning: Y out of Range [0,100]. Got {}. Clamped.",
                self.yy
            )));

            self.yy = self.yy.clamp(0.0, 100.0);
        }
        Ok(warnings)
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {self}
}

impl Verify for Gender {
    fn verify(&mut self, _roots: &mut VerificationRoots, _archive: &mut ZipArchive<Cursor<Bytes>>)
              -> Result<Vec<VerifyWarning>, String>
    {
        let mut warnings = Vec::new();
        if !(-1..=1).contains(&self.0) {
            warnings.push(ConstraintIgnored(format!(
                "Invalid Gender. Must be between one of -1, 0 or 1, got {}", self.0
            )));
        }
        Ok(warnings)
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {self}
}

impl<T: ResourceType + 'static> Verify for Resource<T> {
    fn verify(&mut self, _roots: &mut VerificationRoots, archive: &mut ZipArchive<Cursor<Bytes>>)
              -> Result<Vec<VerifyWarning>, String>
    {
        match self.get_file(archive) {
            Err(UnzipError(e)) => Err(format!("Unzip error: {}", e)),
            Err(NoFileFoundError) => Ok(vec![FallbackUsed(format!("File {} not found. Using default", self.0))]),
            Ok(_) => Ok(Vec::new())
        }
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {self}
}

impl<T: Verify + 'static> Verify for NodePath<T> where FixtureType: ResolveNodePath<T> {

    fn verify(&mut self, roots: &mut VerificationRoots, _archive: &mut ZipArchive<Cursor<Bytes>>)
              -> Result<Vec<VerifyWarning>, String>
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
              -> Result<Vec<VerifyWarning>, String>
    {
        let mut warnings = Vec::new();

        if let Some(path) = &mut self.0 {
            match path.verify(roots, archive) {
                Ok(path_warnings) => {
                    warnings.extend(path_warnings);
                }
                Err(e) => {
                    warnings.push(FallbackUsed(format!(
                        "Optional NodePath reference failed validation and was removed. Original error: {}", e
                    )));

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
              -> Result<Vec<VerifyWarning>, String>
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
              -> Result<Vec<VerifyWarning>, String>
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
              -> Result<Vec<VerifyWarning>, String>
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
              -> Result<Vec<VerifyWarning>, String>
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
