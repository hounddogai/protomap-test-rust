use super::*;

use acme_kit::text;

/// Where a primary retention metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryRetentionMetricStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PrimaryRetentionMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PrimaryRetentionMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary retention metric as the retention team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRetentionMetric {
    pub code: String,
    pub name: String,
    pub quota_units: i64,
    pub maximum: f64,
    pub phone: String,
    pub pinned: bool,
    pub country: String,
    pub checksum: String,
    pub created_at: i64,
    pub budget: f64,
    pub capacity: i64,
    pub status: PrimaryRetentionMetricStatus,
}

impl PrimaryRetentionMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryRetentionMetricStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryRetentionMetricStatus) -> bool {
        match self.status {
            PrimaryRetentionMetricStatus::Proposed => {
                return next == PrimaryRetentionMetricStatus::Approved
                    || next == PrimaryRetentionMetricStatus::Rejected;
            }
            PrimaryRetentionMetricStatus::Approved => {
                return next == PrimaryRetentionMetricStatus::Rejected
                    || next == PrimaryRetentionMetricStatus::Withdrawn;
            }
            PrimaryRetentionMetricStatus::Rejected => {
                match next {
                    PrimaryRetentionMetricStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PrimaryRetentionMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary retention metric {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.quota_units as f64) * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryRetentionMetric {
        PrimaryRetentionMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            quota_units: self.quota_units,
            maximum: self.maximum,
            phone: self.phone.clone(),
            pinned: self.pinned,
            country: self.country.clone(),
            checksum: self.checksum.clone(),
            created_at: self.created_at,
            budget: self.budget,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a primary retention metric in status proposed.
pub fn new_primary_retention_metric(code: &str, name: &str) -> PrimaryRetentionMetric {
    PrimaryRetentionMetric {
        code: code.to_string(),
        name: name.to_string(),
        quota_units: 0,
        maximum: 0.0,
        phone: "".to_string(),
        pinned: false,
        country: "".to_string(),
        checksum: "".to_string(),
        created_at: 0,
        budget: 0.0,
        capacity: 0,
        status: PrimaryRetentionMetricStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_primary_retention_metric("code-345", "name-253");
        assert_eq!(item.status, PrimaryRetentionMetricStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_primary_retention_metric("code-345", "name-253");
        if item.move_to(PrimaryRetentionMetricStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryRetentionMetricStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_primary_retention_metric("code-345", "name-253");
        item.status = PrimaryRetentionMetricStatus::Withdrawn;
        assert!(
            item.move_to(PrimaryRetentionMetricStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
