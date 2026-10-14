use super::*;

use acme_kit::text;

/// Where a weighted alert batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedAlertBatchStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl WeightedAlertBatchStatus {
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

impl std::fmt::Display for WeightedAlertBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted alert batch. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertBatch {
    pub code: String,
    pub label: String,
    pub country: String,
    pub checksum: String,
    pub quota_units: i64,
    pub tenant_id: String,
    pub external_ref: String,
    pub retries: i64,
    pub created_at: i64,
    pub status: WeightedAlertBatchStatus,
}

impl WeightedAlertBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedAlertBatchStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedAlertBatchStatus) -> bool {
        match self.status {
            WeightedAlertBatchStatus::Open => {
                match next {
                    WeightedAlertBatchStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedAlertBatchStatus::InReview => {
                match next {
                    WeightedAlertBatchStatus::Resolved | WeightedAlertBatchStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedAlertBatchStatus::Resolved => {
                return next == WeightedAlertBatchStatus::Reopened || next == WeightedAlertBatchStatus::Archived;
            }
            WeightedAlertBatchStatus::Reopened => {
                match next {
                    WeightedAlertBatchStatus::Archived => {
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

    pub fn move_to(&mut self, next: WeightedAlertBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted alert batch {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.retries as f64) * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedAlertBatch {
        WeightedAlertBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            country: self.country.clone(),
            checksum: self.checksum.clone(),
            quota_units: self.quota_units,
            tenant_id: self.tenant_id.clone(),
            external_ref: self.external_ref.clone(),
            retries: self.retries,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a weighted alert batch in status open.
pub fn new_weighted_alert_batch(code: &str, label: &str) -> WeightedAlertBatch {
    WeightedAlertBatch {
        code: code.to_string(),
        label: label.to_string(),
        country: "".to_string(),
        checksum: "".to_string(),
        quota_units: 0,
        tenant_id: "".to_string(),
        external_ref: "".to_string(),
        retries: 0,
        created_at: 0,
        status: WeightedAlertBatchStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_weighted_alert_batch("code-459", "label-373");
        assert_eq!(item.status, WeightedAlertBatchStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_weighted_alert_batch("code-459", "label-373");
        if item.move_to(WeightedAlertBatchStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedAlertBatchStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_weighted_alert_batch("code-459", "label-373");
        item.status = WeightedAlertBatchStatus::Archived;
        assert!(
            item.move_to(WeightedAlertBatchStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
