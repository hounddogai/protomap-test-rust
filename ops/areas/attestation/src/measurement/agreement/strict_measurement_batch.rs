use super::*;

use acme_kit::text;

/// Where a strict measurement batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictMeasurementBatchStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl StrictMeasurementBatchStatus {
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

impl std::fmt::Display for StrictMeasurementBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict measurement batch and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictMeasurementBatch {
    pub code: String,
    pub title: String,
    pub created_at: i64,
    pub tenant_id: String,
    pub host: String,
    pub phone: String,
    pub quota_units: i64,
    pub status: StrictMeasurementBatchStatus,
}

impl StrictMeasurementBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictMeasurementBatchStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictMeasurementBatchStatus) -> bool {
        match self.status {
            StrictMeasurementBatchStatus::Open => {
                match next {
                    StrictMeasurementBatchStatus::InReview | StrictMeasurementBatchStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictMeasurementBatchStatus::InReview => {
                match next {
                    StrictMeasurementBatchStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictMeasurementBatchStatus::Resolved => {
                return next == StrictMeasurementBatchStatus::Reopened
                    || next == StrictMeasurementBatchStatus::Archived;
            }
            StrictMeasurementBatchStatus::Reopened => {
                match next {
                    StrictMeasurementBatchStatus::Archived => {
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

    pub fn move_to(&mut self, next: StrictMeasurementBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict measurement batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        ((self.quota_units as f64 - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictMeasurementBatch {
        StrictMeasurementBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            created_at: self.created_at,
            tenant_id: self.tenant_id.clone(),
            host: self.host.clone(),
            phone: self.phone.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a strict measurement batch in status open.
pub fn new_strict_measurement_batch(code: &str, title: &str) -> StrictMeasurementBatch {
    StrictMeasurementBatch {
        code: code.to_string(),
        title: title.to_string(),
        created_at: 0,
        tenant_id: "".to_string(),
        host: "".to_string(),
        phone: "".to_string(),
        quota_units: 0,
        status: StrictMeasurementBatchStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_strict_measurement_batch("code-739", "title-851");
        assert_eq!(item.status, StrictMeasurementBatchStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_strict_measurement_batch("code-739", "title-851");
        if item.move_to(StrictMeasurementBatchStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictMeasurementBatchStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_strict_measurement_batch("code-739", "title-851");
        item.status = StrictMeasurementBatchStatus::Archived;
        assert!(
            item.move_to(StrictMeasurementBatchStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
