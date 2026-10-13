//! US Core vital sign Observation checks.
//!
//! Focuses on problems that still show up after schema validation: BP components,
//! vital-signs category, UCUM, and LOINC/code mismatches (valid JSON, wrong clinical meaning).

use serde_json::Value;

use crate::outcome::{ValidationIssue, ValidationReport};
use crate::profile::{
    LOINC_BP_PANEL, LOINC_BODY_WEIGHT, LOINC_DIASTOLIC, LOINC_HEART_RATE, LOINC_SYSTOLIC,
    LOINC_SYSTEM, OBS_CATEGORY_SYSTEM, UCUM_SYSTEM, US_CORE_BLOOD_PRESSURE_PROFILE,
    US_CORE_BODY_WEIGHT_PROFILE, US_CORE_HEART_RATE_PROFILE, VITAL_SIGNS_CATEGORY_CODE,
};

pub fn validate_observation_json(json: &str) -> Result<ValidationReport, String> {
    let value: Value = serde_json::from_str(json).map_err(|e| format!("JSON parse: {e}"))?;
    Ok(validate_observation_value(&value))
}

pub fn validate_observation_value(obs: &Value) -> ValidationReport {
    let mut report = ValidationReport::ok();

    if obs.get("resourceType").and_then(Value::as_str) != Some("Observation") {
        report.push(ValidationIssue::error(
            "Observation.resourceType",
            "resourceType must be Observation",
            "observation-resource-type",
        ));
        return report;
    }

    if obs.get("status").and_then(Value::as_str).is_none() {
        report.push(ValidationIssue::error(
            "Observation.status",
            "status is required",
            "observation-status",
        ));
    }

    check_vital_signs_category(obs, &mut report);
    check_code_loinc(obs, &mut report);

    let profiles = meta_profiles(obs);
    let is_bp = profiles.iter().any(|p| p == US_CORE_BLOOD_PRESSURE_PROFILE);
    let is_hr = profiles.iter().any(|p| p == US_CORE_HEART_RATE_PROFILE);
    let is_weight = profiles.iter().any(|p| p == US_CORE_BODY_WEIGHT_PROFILE);

    if is_bp {
        check_meta_profile(obs, US_CORE_BLOOD_PRESSURE_PROFILE, &mut report);
        check_blood_pressure_shape(obs, &mut report);
    } else if is_hr {
        check_meta_profile(obs, US_CORE_HEART_RATE_PROFILE, &mut report);
        check_simple_vital_quantity(obs, "/valueQuantity", "/min", false, &mut report);
        check_heart_rate_semantics(obs, &mut report);
    } else if is_weight {
        check_meta_profile(obs, US_CORE_BODY_WEIGHT_PROFILE, &mut report);
        check_simple_vital_quantity(obs, "/valueQuantity", "kg", true, &mut report);
        check_body_weight_semantics(obs, &mut report);
    } else {
        report.push(ValidationIssue::warning(
            "Observation.meta.profile",
            "expected us-core-blood-pressure, us-core-heart-rate, or us-core-body-weight profile for this validator",
            "us-core-vital-profile",
        ));
    }

    report
}

fn meta_profiles(obs: &Value) -> Vec<String> {
    obs.pointer("/meta/profile")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}

fn check_meta_profile(obs: &Value, expected: &str, report: &mut ValidationReport) {
    if !meta_profiles(obs).iter().any(|p| p == expected) {
        report.push(ValidationIssue::error(
            "Observation.meta.profile",
            format!("meta.profile should include {}", expected),
            "us-core-meta-profile",
        ));
    }
}

fn check_vital_signs_category(obs: &Value, report: &mut ValidationReport) {
    let categories = obs.get("category").and_then(Value::as_array);
    let has_vital = categories.is_some_and(|cats| {
        cats.iter().any(|cat| {
            cat.get("coding")
                .and_then(Value::as_array)
                .is_some_and(|codings| {
                    codings.iter().any(|c| {
                        c.get("system").and_then(Value::as_str) == Some(OBS_CATEGORY_SYSTEM)
                            && c.get("code").and_then(Value::as_str)
                                == Some(VITAL_SIGNS_CATEGORY_CODE)
                    })
                })
        })
    });

    if !has_vital {
        report.push(ValidationIssue::error(
            "Observation.category",
            format!(
                "vital signs need category {}|{}",
                OBS_CATEGORY_SYSTEM,
                VITAL_SIGNS_CATEGORY_CODE
            ),
            "vital-signs-category",
        ));
    }
}

fn check_code_loinc(obs: &Value, report: &mut ValidationReport) {
    let codings = obs.pointer("/code/coding").and_then(Value::as_array);
    let has_loinc = codings.is_some_and(|c| {
        c.iter().any(|coding| {
            coding.get("system").and_then(Value::as_str) == Some(LOINC_SYSTEM)
                && coding
                    .get("code")
                    .and_then(Value::as_str)
                    .is_some_and(|code| !code.is_empty())
        })
    });

    if !has_loinc {
        report.push(ValidationIssue::error(
            "Observation.code",
            "Observation.code needs a LOINC coding (system http://loinc.org)",
            "code-loinc-required",
        ));
    }
}

fn loinc_code_at(obs: &Value, pointer: &str) -> Option<String> {
    obs.pointer(pointer)
        .and_then(Value::as_array)
        .and_then(|codings| {
            codings
                .iter()
                .find(|c| c.get("system").and_then(Value::as_str) == Some(LOINC_SYSTEM))
                .and_then(|c| c.get("code").and_then(Value::as_str))
                .map(String::from)
        })
}

fn check_ucum_quantity(obs: &Value, path: &str, report: &mut ValidationReport) {
    let qty = obs.pointer(path);
    if qty.is_none() {
        return;
    }
    let system = qty.and_then(|q| q.get("system")).and_then(Value::as_str);
    if system != Some(UCUM_SYSTEM) {
        report.push(ValidationIssue::error(
            format!("Observation{path}.system"),
            format!(
                "quantity system should be {} for UCUM (got {:?})",
                UCUM_SYSTEM,
                system
            ),
            "ucum-system",
        ));
    }
}

fn check_simple_vital_quantity(
    obs: &Value,
    qty_path: &str,
    expected_unit: &str,
    allow_alternate_mass_units: bool,
    report: &mut ValidationReport,
) {
    if obs.pointer(qty_path).is_none() {
        report.push(ValidationIssue::error(
            format!("Observation{}", qty_path),
            "valueQuantity is required for this vital sign",
            "value-quantity-required",
        ));
        return;
    }
    check_ucum_quantity(obs, qty_path, report);
    let unit = obs
        .pointer(qty_path)
        .and_then(|q| q.get("unit"))
        .and_then(Value::as_str);
    let code = obs
        .pointer(qty_path)
        .and_then(|q| q.get("code"))
        .and_then(Value::as_str);

    let unit_ok = unit == Some(expected_unit) || code == Some(expected_unit);
    if allow_alternate_mass_units && (unit == Some("[lb_av]") || code == Some("[lb_av]")) {
        return;
    }
    if !unit_ok {
        report.push(ValidationIssue::warning(
            format!("Observation{}", qty_path),
            format!("expected unit {} for this vital (got unit {:?}, code {:?})", expected_unit, unit, code),
            "ucum-unit-expected",
        ));
    }
}

// US Core BP is a panel: components for systolic and diastolic, not a single valueQuantity on the parent.
fn check_blood_pressure_shape(obs: &Value, report: &mut ValidationReport) {
    if obs.get("valueQuantity").is_some() {
        report.push(ValidationIssue::error(
            "Observation.valueQuantity",
            "blood pressure uses component observations, not valueQuantity on the parent",
            "bp-no-parent-value",
        ));
    }

    let panel_code = loinc_code_at(obs, "/code/coding");
    if panel_code.as_deref() != Some(LOINC_BP_PANEL) {
        report.push(ValidationIssue::error(
            "Observation.code",
            format!("blood pressure panel code should be LOINC {}", LOINC_BP_PANEL),
            "bp-panel-loinc",
        ));
    }

    let components = obs.get("component").and_then(Value::as_array);
    let Some(components) = components else {
        report.push(ValidationIssue::error(
            "Observation.component",
            "blood pressure requires component entries for systolic and diastolic",
            "bp-components-required",
        ));
        return;
    };

    let mut has_sys = false;
    let mut has_dia = false;

    for (idx, _comp) in components.iter().enumerate() {
        let base = format!("/component/{}", idx);
        let code = loinc_code_at(obs, &format!("{base}/code/coding"));
        match code.as_deref() {
            Some(LOINC_SYSTOLIC) => {
                has_sys = true;
                check_ucum_quantity(obs, &format!("{base}/valueQuantity"), report);
                check_bp_component_unit(obs, &format!("{base}/valueQuantity"), report);
            }
            Some(LOINC_DIASTOLIC) => {
                has_dia = true;
                check_ucum_quantity(obs, &format!("{base}/valueQuantity"), report);
                check_bp_component_unit(obs, &format!("{base}/valueQuantity"), report);
            }
            Some(other) => {
                report.push(ValidationIssue::warning(
                    format!("Observation.component[{}].code", idx),
                    format!("unexpected component LOINC {} for US Core BP", other),
                    "bp-component-loinc",
                ));
            }
            None => {
                report.push(ValidationIssue::error(
                    format!("Observation.component[{}].code", idx),
                    "each BP component needs a LOINC code",
                    "bp-component-loinc",
                ));
            }
        }
    }

    if !has_sys || !has_dia {
        report.push(ValidationIssue::error(
            "Observation.component",
            format!(
                "need systolic ({}) and diastolic ({}) components",
                LOINC_SYSTOLIC,
                LOINC_DIASTOLIC
            ),
            "bp-systolic-diastolic",
        ));
    }
}

fn check_bp_component_unit(obs: &Value, qty_path: &str, report: &mut ValidationReport) {
    let unit = obs
        .pointer(qty_path)
        .and_then(|q| q.get("unit"))
        .and_then(Value::as_str);
    let code = obs
        .pointer(qty_path)
        .and_then(|q| q.get("code"))
        .and_then(Value::as_str);
    if unit != Some("mm[Hg]") && code != Some("mm[Hg]") {
        report.push(ValidationIssue::error(
            format!("Observation{qty_path}"),
            "BP components should use UCUM mm[Hg]",
            "bp-ucum-mmhg",
        ));
    }
}

// Classic production bug: mapper reuses the wrong LOINC constant; value looks plausible.
fn check_heart_rate_semantics(obs: &Value, report: &mut ValidationReport) {
    let code = loinc_code_at(obs, "/code/coding");
    if code.as_deref() != Some(LOINC_HEART_RATE) {
        report.push(ValidationIssue::error(
            "Observation.code",
            format!(
                "heart rate profile expects LOINC {} (got {:?})",
                LOINC_HEART_RATE,
                code
            ),
            "heart-rate-loinc-mismatch",
        ));
    }
    if code.as_deref() == Some(LOINC_BODY_WEIGHT) {
        report.push(ValidationIssue::error(
            "Observation.code",
            "body weight LOINC on a heart rate observation — common mapper copy/paste failure",
            "semantic-loinc-wrong-vital",
        ));
    }
    check_ucum_quantity(obs, "/valueQuantity", report);
    let unit = obs
        .pointer("/valueQuantity/unit")
        .and_then(Value::as_str);
    let ucode = obs
        .pointer("/valueQuantity/code")
        .and_then(Value::as_str);
    if unit != Some("/min") && ucode != Some("/min") {
        report.push(ValidationIssue::warning(
            "Observation.valueQuantity",
            "heart rate usually uses UCUM /min",
            "heart-rate-ucum",
        ));
    }
}

fn check_body_weight_semantics(obs: &Value, report: &mut ValidationReport) {
    let code = loinc_code_at(obs, "/code/coding");
    if code.as_deref() != Some(LOINC_BODY_WEIGHT) {
        report.push(ValidationIssue::error(
            "Observation.code",
            format!(
                "body weight profile expects LOINC {} (got {:?})",
                LOINC_BODY_WEIGHT,
                code
            ),
            "body-weight-loinc",
        ));
    }
}
