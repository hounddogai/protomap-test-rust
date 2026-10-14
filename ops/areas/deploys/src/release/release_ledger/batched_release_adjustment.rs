use super::*;

use acme_kit::text;

/// Where a batched release adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedReleaseAdjustmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BatchedReleaseAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BatchedReleaseAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched release adjustment as the release team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReleaseAdjustment {
    pub code: String,
    pub label: String,
    pub ttl_seconds: i64,
    pub expires_at: i64,
    pub maximum: f64,
    pub quantity: i64,
    pub country: String,
    pub contact_email: String,
    pub starts_at: i64,
    pub retries: i64,
    pub host: String,
    pub status: BatchedReleaseAdjustmentStatus,
}

impl BatchedReleaseAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedReleaseAdjustmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedReleaseAdjustmentStatus) -> bool {
        match self.status {
            BatchedReleaseAdjustmentStatus::Proposed => {
                return next == BatchedReleaseAdjustmentStatus::Approved;
            }
            BatchedReleaseAdjustmentStatus::Approved => {
                match next {
                    BatchedReleaseAdjustmentStatus::Rejected | BatchedReleaseAdjustmentStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedReleaseAdjustmentStatus::Rejected => {
                match next {
                    BatchedReleaseAdjustmentStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: BatchedReleaseAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched release adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.retries as f64 + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedReleaseAdjustment {
        BatchedReleaseAdjustment {
            code: self.code.clone(),
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            expires_at: self.expires_at,
            maximum: self.maximum,
            quantity: self.quantity,
            country: self.country.clone(),
            contact_email: self.contact_email.clone(),
            starts_at: self.starts_at,
            retries: self.retries,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a batched release adjustment in status proposed.
pub fn new_batched_release_adjustment(code: &str, label: &str) -> BatchedReleaseAdjustment {
    BatchedReleaseAdjustment {
        code: code.to_string(),
        label: label.to_string(),
        ttl_seconds: 0,
        expires_at: 0,
        maximum: 0.0,
        quantity: 0,
        country: "".to_string(),
        contact_email: "".to_string(),
        starts_at: 0,
        retries: 0,
        host: "".to_string(),
        status: BatchedReleaseAdjustmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_batched_release_adjustment("code-816", "label-779");
        assert_eq!(item.status, BatchedReleaseAdjustmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_batched_release_adjustment("code-816", "label-779");
        if item.move_to(BatchedReleaseAdjustmentStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedReleaseAdjustmentStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_batched_release_adjustment("code-816", "label-779");
        item.status = BatchedReleaseAdjustmentStatus::Withdrawn;
        assert!(
            item.move_to(BatchedReleaseAdjustmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
