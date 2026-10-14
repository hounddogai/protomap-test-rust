use super::*;

use acme_kit::text;

/// Where a manual measurement exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualMeasurementExceptionStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ManualMeasurementExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for ManualMeasurementExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual measurement exception. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualMeasurementException {
    pub code: String,
    pub title: String,
    pub host: String,
    pub minimum: f64,
    pub external_ref: String,
    pub key_id: String,
    pub budget: f64,
    pub burst: i64,
    pub priority: i64,
    pub discount: f64,
    pub country: String,
    pub status: ManualMeasurementExceptionStatus,
}

impl ManualMeasurementException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualMeasurementExceptionStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualMeasurementExceptionStatus) -> bool {
        match self.status {
            ManualMeasurementExceptionStatus::Open => {
                return next == ManualMeasurementExceptionStatus::InReview
                    || next == ManualMeasurementExceptionStatus::Reopened;
            }
            ManualMeasurementExceptionStatus::InReview => {
                return next == ManualMeasurementExceptionStatus::Archived;
            }
            ManualMeasurementExceptionStatus::Resolved => {
                return next == ManualMeasurementExceptionStatus::Reopened;
            }
            ManualMeasurementExceptionStatus::Reopened => {
                return next == ManualMeasurementExceptionStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualMeasurementExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual measurement exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.budget + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualMeasurementException {
        ManualMeasurementException {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            minimum: self.minimum,
            external_ref: self.external_ref.clone(),
            key_id: self.key_id.clone(),
            budget: self.budget,
            burst: self.burst,
            priority: self.priority,
            discount: self.discount,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a manual measurement exception in status open.
pub fn new_manual_measurement_exception(code: &str, title: &str) -> ManualMeasurementException {
    ManualMeasurementException {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        minimum: 0.0,
        external_ref: "".to_string(),
        key_id: "".to_string(),
        budget: 0.0,
        burst: 0,
        priority: 0,
        discount: 0.0,
        country: "".to_string(),
        status: ManualMeasurementExceptionStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_manual_measurement_exception("code-434", "title-889");
        assert_eq!(item.status, ManualMeasurementExceptionStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_manual_measurement_exception("code-434", "title-889");
        if item.move_to(ManualMeasurementExceptionStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualMeasurementExceptionStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_manual_measurement_exception("code-434", "title-889");
        item.status = ManualMeasurementExceptionStatus::Archived;
        assert!(
            item.move_to(ManualMeasurementExceptionStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
