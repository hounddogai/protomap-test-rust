use super::*;

use acme_kit::text;

/// Where a weighted ttl policy threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedTtlPolicyThresholdStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl WeightedTtlPolicyThresholdStatus {
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

impl std::fmt::Display for WeightedTtlPolicyThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted ttl policy threshold as the ttl policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTtlPolicyThreshold {
    pub code: String,
    pub label: String,
    pub checksum: String,
    pub ends_at: i64,
    pub source: String,
    pub capacity: i64,
    pub rate: f64,
    pub external_ref: String,
    pub ratio: f64,
    pub locale: String,
    pub sequence: i64,
    pub status: WeightedTtlPolicyThresholdStatus,
}

impl WeightedTtlPolicyThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedTtlPolicyThresholdStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedTtlPolicyThresholdStatus) -> bool {
        match self.status {
            WeightedTtlPolicyThresholdStatus::Open => {
                match next {
                    WeightedTtlPolicyThresholdStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedTtlPolicyThresholdStatus::InReview => {
                match next {
                    WeightedTtlPolicyThresholdStatus::Reopened | WeightedTtlPolicyThresholdStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedTtlPolicyThresholdStatus::Resolved => {
                return next == WeightedTtlPolicyThresholdStatus::Reopened
                    || next == WeightedTtlPolicyThresholdStatus::Archived;
            }
            WeightedTtlPolicyThresholdStatus::Reopened => {
                return next == WeightedTtlPolicyThresholdStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedTtlPolicyThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted ttl policy threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.capacity as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedTtlPolicyThreshold {
        WeightedTtlPolicyThreshold {
            code: self.code.clone(),
            label: self.label.clone(),
            checksum: self.checksum.clone(),
            ends_at: self.ends_at,
            source: self.source.clone(),
            capacity: self.capacity,
            rate: self.rate,
            external_ref: self.external_ref.clone(),
            ratio: self.ratio,
            locale: self.locale.clone(),
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a weighted ttl policy threshold in status open.
pub fn new_weighted_ttl_policy_threshold(code: &str, label: &str) -> WeightedTtlPolicyThreshold {
    WeightedTtlPolicyThreshold {
        code: code.to_string(),
        label: label.to_string(),
        checksum: "".to_string(),
        ends_at: 0,
        source: "".to_string(),
        capacity: 0,
        rate: 0.0,
        external_ref: "".to_string(),
        ratio: 0.0,
        locale: "".to_string(),
        sequence: 0,
        status: WeightedTtlPolicyThresholdStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_weighted_ttl_policy_threshold("code-496", "label-717");
        assert_eq!(item.status, WeightedTtlPolicyThresholdStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_weighted_ttl_policy_threshold("code-496", "label-717");
        if item.move_to(WeightedTtlPolicyThresholdStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedTtlPolicyThresholdStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_weighted_ttl_policy_threshold("code-496", "label-717");
        item.status = WeightedTtlPolicyThresholdStatus::Archived;
        assert!(
            item.move_to(WeightedTtlPolicyThresholdStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
