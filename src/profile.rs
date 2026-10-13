//! US Core vital sign profiles and codes (STU 6.1).

pub const US_CORE_BLOOD_PRESSURE_PROFILE: &str =
    "http://hl7.org/fhir/us/core/StructureDefinition/us-core-blood-pressure";

pub const US_CORE_HEART_RATE_PROFILE: &str =
    "http://hl7.org/fhir/us/core/StructureDefinition/us-core-heart-rate";

pub const US_CORE_BODY_WEIGHT_PROFILE: &str =
    "http://hl7.org/fhir/us/core/StructureDefinition/us-core-body-weight";

pub const LOINC_SYSTEM: &str = "http://loinc.org";

pub const UCUM_SYSTEM: &str = "http://unitsofmeasure.org";

pub const OBS_CATEGORY_SYSTEM: &str =
    "http://terminology.hl7.org/CodeSystem/observation-category";

pub const VITAL_SIGNS_CATEGORY_CODE: &str = "vital-signs";

/// Blood pressure panel — parent Observation.code when using components.
pub const LOINC_BP_PANEL: &str = "85354-9";

pub const LOINC_SYSTOLIC: &str = "8480-6";
pub const LOINC_DIASTOLIC: &str = "8462-4";
pub const LOINC_HEART_RATE: &str = "8867-4";
pub const LOINC_BODY_WEIGHT: &str = "29463-7";
