//! Validation issues shaped like OperationOutcome diagnostics.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    Error,
    Warning,
    Information,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationIssue {
    pub severity: IssueSeverity,
    pub path: String,
    pub detail: String,
    pub rule_id: Option<String>,
}

impl ValidationIssue {
    pub fn error(path: impl Into<String>, detail: impl Into<String>, rule_id: &str) -> Self {
        Self {
            severity: IssueSeverity::Error,
            path: path.into(),
            detail: detail.into(),
            rule_id: Some(rule_id.into()),
        }
    }

    pub fn warning(path: impl Into<String>, detail: impl Into<String>, rule_id: &str) -> Self {
        Self {
            severity: IssueSeverity::Warning,
            path: path.into(),
            detail: detail.into(),
            rule_id: Some(rule_id.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationReport {
    pub issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    pub fn ok() -> Self {
        Self { issues: Vec::new() }
    }

    pub fn push(&mut self, issue: ValidationIssue) {
        self.issues.push(issue);
    }

    pub fn is_valid(&self) -> bool {
        !self
            .issues
            .iter()
            .any(|i| i.severity == IssueSeverity::Error)
    }
}
