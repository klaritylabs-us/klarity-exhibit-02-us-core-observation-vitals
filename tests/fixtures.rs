use std::fs;
use std::path::PathBuf;

use us_core_observation_vitals::validate_observation_json;

fn fixture(name: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("examples");
    path.push(name);
    fs::read_to_string(path).expect("fixture")
}

#[test]
fn valid_blood_pressure() {
    let report = validate_observation_json(&fixture("valid-blood-pressure.json")).unwrap();
    assert!(report.is_valid(), "{:?}", report.issues);
}

#[test]
fn valid_heart_rate() {
    let report = validate_observation_json(&fixture("valid-heart-rate.json")).unwrap();
    assert!(report.is_valid(), "{:?}", report.issues);
}

#[test]
fn bp_single_value_fails() {
    let report = validate_observation_json(&fixture("invalid-bp-single-value.json")).unwrap();
    assert!(!report.is_valid());
}

#[test]
fn heart_rate_wrong_loinc_fails() {
    let report = validate_observation_json(&fixture("invalid-heart-rate-wrong-loinc.json")).unwrap();
    assert!(!report.is_valid());
}

#[test]
fn missing_vital_category_fails() {
    let report = validate_observation_json(&fixture("invalid-missing-vital-category.json")).unwrap();
    assert!(!report.is_valid());
}
