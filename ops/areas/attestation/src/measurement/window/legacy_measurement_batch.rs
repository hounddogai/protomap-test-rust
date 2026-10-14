use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a legacy measurement batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyMeasurementBatchStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LegacyMeasurementBatchStatus {
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

impl std::fmt::Display for LegacyMeasurementBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy measurement batch as the measurement team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMeasurementBatch {
    pub code: String,
    pub name: String,
    pub attributes: BTreeMap<String, String>,
    pub phone: String,
    pub summary: String,
    pub floor_value: f64,
    pub minimum: f64,
    pub version: i64,
    pub approved: bool,
    pub grace_days: i64,
    pub status: LegacyMeasurementBatchStatus,
}

impl LegacyMeasurementBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyMeasurementBatchStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyMeasurementBatchStatus) -> bool {
        match self.status {
            LegacyMeasurementBatchStatus::Open => {
                return next == LegacyMeasurementBatchStatus::InReview;
            }
            LegacyMeasurementBatchStatus::InReview => {
                return next == LegacyMeasurementBatchStatus::Resolved
                    || next == LegacyMeasurementBatchStatus::Reopened;
            }
            LegacyMeasurementBatchStatus::Resolved => {
                match next {
                    LegacyMeasurementBatchStatus::Reopened | LegacyMeasurementBatchStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyMeasurementBatchStatus::Reopened => {
                match next {
                    LegacyMeasurementBatchStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyMeasurementBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy measurement batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy measurement batch", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn net_exposure(&self) -> f64 {
        (self.minimum * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyMeasurementBatch {
        LegacyMeasurementBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            attributes: self.attributes.clone(),
            phone: self.phone.clone(),
            summary: self.summary.clone(),
            floor_value: self.floor_value,
            minimum: self.minimum,
            version: self.version,
            approved: self.approved,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a legacy measurement batch in status open.
pub fn new_legacy_measurement_batch(code: &str, name: &str) -> LegacyMeasurementBatch {
    LegacyMeasurementBatch {
        code: code.to_string(),
        name: name.to_string(),
        attributes: BTreeMap::new(),
        phone: "".to_string(),
        summary: "".to_string(),
        floor_value: 0.0,
        minimum: 0.0,
        version: 0,
        approved: false,
        grace_days: 0,
        status: LegacyMeasurementBatchStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_legacy_measurement_batch("code-14", "name-809");
        assert_eq!(item.status, LegacyMeasurementBatchStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_legacy_measurement_batch("code-14", "name-809");
        if item.move_to(LegacyMeasurementBatchStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyMeasurementBatchStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_legacy_measurement_batch("code-14", "name-809");
        item.status = LegacyMeasurementBatchStatus::Archived;
        assert!(
            item.move_to(LegacyMeasurementBatchStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
